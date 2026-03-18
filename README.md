# RLMX — RuVix Cognition Kernel

RLMX is an OS-kernel-inspired runtime for LLM agents. Instead of giving agents direct API access, it exposes **12 capability-secured syscall primitives** that agents call through a structured interface. Written in Rust, async on Tokio.

Now expanded into a **distributed AI agent swarm** with 12 specialized agent types, layered consensus (PBFT/Raft/Gossip), neural model routing, tiered inference, and browser WASM compute pooling.

## What It Does

- **Vector memory** with cosine similarity search and Hot/Warm/Cold tiering
- **Property graph** with Cypher queries, min-cut, and heat-kernel diffusion
- **Capability-secured processes** with HMAC-SHA256 tokens and hierarchical derivation
- **Witness-chained audit log** (SHA-256 append-only proof)
- **MCP server** (23 JSON-RPC 2.0 tools) over HTTP + WebSocket
- **12 typed agent roles** with a 12x12 permission matrix (ADR-005)
- **Distributed swarm** with 4-zone hierarchical topology (ADR-001)
- **Layered consensus**: PBFT (critical), Raft (metadata), Gossip (health) (ADR-002)
- **Neural model routing** via FastGRNN with 14-feature input and online learning (ADR-003)
- **Tiered inference** with confidence-based escalation: Small -> Medium -> Cloud (ADR-004)
- **Evolutionary auto-research** with mutation, cross-pollination, cloud escalation (ADR-006)
- **WebSocket real-time events** with typed SwarmEvent streaming (ADR-008)
- **Browser WASM compute pool** with priority queuing and fault tolerance (ADR-009)
- **Edge inference** via GGUF models on CPU/Metal/CUDA
- **Browser inference** via WebAssembly (WebGPU/SIMD)

## Architecture

```
                  rlmx-cli (binary)
                    ├── rlmx-kernel     (core: 12 syscalls, router, events)
                    ├── rlmx-mcp        (23 tools, HTTP :3000, WS :3001)
                    ├── rlmx-swarm      (4 zones, PBFT/Raft/Gossip, browser pool)
                    ├── rlmx-agents     (12 agent types, permissions, research)
                    ├── rlmx-ruvllm     (tiered engine, MLX bridge, GGUF)
                    ├── rlmx-rlm        (recursive LLM, vLLM client)
                    ├── rlmx-trm        (tiny neural net, 3-stream)
                    ├── rlmx-rvf        (sealed containers, COW branching)
                    ├── rlmx-plugin     (DomainPlugin trait, safety engine)
                    └── rlmx-cognitive   (SONA, DAG optimizer, nervous system)
```

**Inference paths:** Cloud (vLLM) | Edge (CandleBackend/GGUF) | Tiered (Small->Medium->Remote) | Browser (WebGPU/WASM SIMD) | Swarm (cross-zone scatter-gather)

## Quick Start

```bash
# Build (11 crates)
cargo build --workspace

# Test (462 tests, 0 failures)
cargo test --workspace

# Lint (0 warnings)
cargo clippy --workspace -- -D warnings

# Start MCP server + frontend
cargo run -p rlmx-cli -- serve --port 3000   # Terminal 1
cd frontend && python3 -m http.server 8080    # Terminal 2
# Open http://127.0.0.1:8080 — dashboard works with or without server
```

### With Edge Inference

```bash
mkdir -p ~/.rlmx/models
# Download GGUF model + tokenizer, then:
RLMX_EDGE_MODEL=tinyllama-1.1b-q4_k_m \
  cargo run -p rlmx-cli --features "ruvllm,metal" -- serve --port 3000
```

### Multi-Tab / Distributed Mode

Open multiple browser tabs at `http://127.0.0.1:8080` — they coordinate via BroadcastChannel. For distributed deployment across machines:

```bash
# Edge node (RPi5 / NUC)
cargo build --release --target aarch64-unknown-linux-gnu -p rlmx-cli --features ruvllm
scp target/aarch64-unknown-linux-gnu/release/rlmx-cli edge-node:
ssh edge-node './rlmx-cli serve --port 3000'

# Cloud burst via SkyPilot (future)
# sky launch -c rlmx-gpu --gpus A100 -- 'rlmx-cli serve --port 3000'

# HuggingFace Jobs (future)
# huggingface-cli jobs submit --image rlmx:latest -- rlmx-cli train --backend cuda
```

## Workspace Crates (11)

| Crate | Purpose |
|-------|---------|
| `rlmx-kernel` | 12-syscall dispatch, vector memory, graph, capability tokens, TinyDancer router, domain events |
| `rlmx-mcp` | 23 JSON-RPC tools, stdio/HTTP, WebSocket events, RBAC |
| `rlmx-swarm` | 4-zone cluster, PBFT/Raft/Gossip consensus, browser compute pool, chaos testing |
| `rlmx-agents` | 12 agent types, 12x12 permission matrix, auto-research, mutations, lifecycle |
| `rlmx-rlm` | Recursive LLM: 5-action grammar, vLLM client |
| `rlmx-trm` | Tiny Recursive Model: 2-layer NN, 3-stream, adaptive halting |
| `rlmx-ruvllm` | Tiered engine, CandleBackend, GGUF, MLX bridge |
| `rlmx-rvf` | Sealed containers, Ed25519 signatures, COW branching |
| `rlmx-plugin` | DomainPlugin trait, safety engine, Ericsson RAN |
| `rlmx-cognitive` | SONA adaptation, DAG optimizer, nervous system |
| `rlmx-cli` | CLI: query, serve, swarm, agent, research, edge, seal |

## CLI Commands

```bash
# Core
rlmx serve --port 3000                       # MCP + WS server
rlmx query -i "search" -s auto               # Semantic search
rlmx ingest -p data.txt                      # Ingest into memory

# Swarm
rlmx swarm start --zone A --port 9000        # Start swarm node
rlmx swarm status                             # Cluster health
rlmx swarm topology                           # Zone map
rlmx swarm chaos --type network-partition     # Fault injection

# Agents
rlmx agent spawn --type researcher --task "optimize latency"
rlmx agent list --type worker
rlmx agent kill <agent-id>

# Research
rlmx research start --topic "reduce val_bpb"
rlmx research status <id>
rlmx train --config '{}' --backend mlx
rlmx forecast --metric swarm_health --horizon-hours 24

# Edge
rlmx edge status / models / load / generate

# Container
rlmx seal -p ./data -o sealed.rvf
rlmx branch -n experiment --from sealed.rvf
```

## MCP Tools (23)

### Core (12)
`rlmx_query`, `rlmx_ingest`, `rlmx_memory_stats`, `rlmx_graph_query`, `rlmx_plugin_list`, `rlmx_plugin_action`, `rlmx_strategy_override`, `rlmx_trm_classify`, `rlmx_rvf_seal`, `rlmx_rvf_branch`, `rlmx_witness_chain`, `rlmx_sona_stats`

### Swarm & Agents (11)
`rlmx_swarm_status` (Viewer), `rlmx_swarm_topology` (Viewer), `rlmx_agent_spawn` (Engineer), `rlmx_agent_list` (Operator), `rlmx_agent_terminate` (Admin), `rlmx_research_start` (Engineer), `rlmx_research_status` (Operator), `rlmx_experiment_list` (Viewer), `rlmx_mutation_history` (Viewer), `rlmx_forecast` (Operator), `rlmx_train` (Engineer)

## Agent Types (12)

Coordinator, Researcher, Router, Experimenter, Worker, Monitor, Reviewer, Trainer, Validator, Replicator, Embedder, Analyst — each with scoped capability tokens per the 12x12 permission matrix.

## Swarm Zones

| Zone | Role | Consensus |
|------|------|-----------|
| A (Compute) | Coordination, heavy inference | PBFT |
| B (Inference) | Edge inference, routing | Raft |
| C (Edge) | Telemetry, validation | Gossip |
| D (Burst) | Cloud overflow, GPU training | On-demand |

## Frontend Dashboard (20+ Views)

Single-page dark-themed dashboard (`frontend/index.html`, ~2900 lines vanilla JS) with real-time visualization. Works standalone with rich demo data or connected to the MCP server.

```bash
cd frontend && python3 -m http.server 8080
# Open http://127.0.0.1:8080
```

### Sections

| Section | Views | Key Features |
|---------|-------|-------------|
| **Swarm** | Overview, Topology Map, Health Monitor, Consensus | 5-zone topology canvas, per-node CPU/mem/latency, PBFT/Raft/Gossip live metrics |
| **Agents** | Agent Manager, Permission Matrix, Spawn Hierarchy | Spawn with zone/tier/runtime config, 12x12 permission preview, lifecycle tree |
| **Compute** | WASM Pool, Benchmark, Active Workers | Capability detection, MatMul benchmark, pull-based worker grid |
| **Memory** | Query, Ingest, Stats | Semantic search, 3-tier Hot/Warm/Cold stats |
| **Graph** | Graph Query | Force-directed knowledge graph (12 nodes, 18 edges), drag-interactive, Cypher input |
| **Edge AI** | Inference, Tiered Models, Chat Format | WASM chat formatting, tiered escalation view |
| **Research** | Tracker, Evolution View, Mutation Log | 3 demo experiments with fitness bars, evolutionary pipeline |
| **Events** | Live Stream | Real-time event feed with color-coded type badges |
| **Tools/RPC** | Tool Registry, Raw Console, Request Log | 23 MCP tools, manual JSON-RPC |

### Built-in Features

- **Demo data**: 25 nodes, 8 agents, 3 experiments, 12-node knowledge graph — pre-populated on load
- **SimEngine**: 2-second tick updates metrics, generates events, random-walks node health
- **Sparkline SVG**: Mini time-series charts in metric cards
- **BroadcastChannel**: Multi-tab coordination via `rlmx-swarm` channel
- **Smart fallback**: Tries MCP server first, shows demo data when server returns empty

## Documentation

| Path | Contents |
|------|----------|
| `docs/ADR/` | 10 Architecture Decision Records (ADR-001 through ADR-010) |
| `docs/DDD/` | 7 Domain-Driven Design documents (context map, ubiquitous language) |
| `docs/frontend-dashboard.md` | Dashboard architecture, modules, data flow, how to add views |
| `deploy/README.md` | Edge deployment guide (RPi5, systemd) |
| `CLAUDE.md` | AI assistant rules, build commands, strict rules, key file locations |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RLMX_EDGE_MODEL` | GGUF model name (without `.gguf`) | unset |
| `RLMX_MODEL_DIR` | Directory for `.gguf` + tokenizer files | `~/.rlmx/models` |

## Feature Flags

| Feature | Effect |
|---------|--------|
| `ruvllm` | Enable CandleBackend for GGUF inference |
| `metal` | Apple Metal GPU acceleration (macOS) |
