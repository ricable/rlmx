# RLMX — RuVix Cognition Kernel

A **voice-first cognition kernel** — an OS-kernel-inspired runtime for LLM agents, activated by voice. Provides capability-secured syscall primitives that agents call instead of accessing arbitrary APIs.

**Hybrid Rust + TypeScript** | 19 Rust crates + 13 npm packages | 47 MCP tools | 17 agent types | 6 swarm zones | 1,748+ tests

## What It Does

- **Voice-first pipeline**: On-device STT -> multi-intent decomposition -> agent swarm -> multimodal response
- **17 capability-secured syscalls** with HMAC-SHA256 tokens and hierarchical derivation
- **17 typed agent roles** with a 17x17 permission matrix
- **6-zone distributed swarm** with PBFT/Raft/Gossip consensus
- **47 MCP tools** (JSON-RPC 2.0) over HTTP + WebSocket with 6-role RBAC
- **Neural model routing** via FastGRNN (18-dim) with online learning
- **Tiered inference**: Small (0.5B) -> Medium (MLX 3-8B) -> Cloud (vLLM)
- **Edge inference** via GGUF models on CPU/Metal/CUDA
- **Browser compute** via WebAssembly (WebGPU/WASM SIMD)
- **Personal mesh** with mDNS discovery, privacy anchor pattern, cross-device sync
- **Federated learning** with differential privacy (Laplace e=1.0)
- **Agent marketplace** with 12 life domains, automated security review, 70/30 revenue split
- **Subscription billing** (6 tiers) with capability-token enforcement

## Architecture

```
 Rust (compute — @aix/core NAPI binary)    TypeScript (orchestration — npm packages)
 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━    ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 rlmx-kernel   (router, vectors, proofs)   aix            (CLI — npx aix)
 rlmx-cognitive (SONA, LoRA, GNN)          @aix/mcp-server (47 tools, RBAC, WS)
 rlmx-ruvllm   (Candle, Metal/CUDA)        @aix/agents    (17x17 permissions)
 rlmx-voice    (VAD, STT, TTS)             @aix/swarm     (6 zones, consensus)
 rlmx-phone    (engagement, battery)        @aix/marketplace (registry, reviews)
 rlmx-trm      (recursive NN)              @aix/mesh      (discovery, sync)
 rlmx-rvf      (Ed25519, witness)           @aix/billing   (6 tiers, Stripe)
 rlmx-napi     (NAPI bridge — 31 fns)      @aix/federation (cycles, privacy)
 rlmx-wasm     (browser kernel)             @aix/plugin    (domain plugins)
                                            @aix/rlm       (vLLM client)
                                            @aix/shared    (types, events, errors)
                                            @aix/core      (NAPI loader)
```

## Quick Start

```bash
# === Rust ===
cargo build --workspace              # Build all 19 crates
cargo test --workspace               # Run 946+ Rust tests
cargo clippy --workspace -- -D warnings

# === TypeScript ===
npm install                          # Install workspace dependencies
npm run test:ts                      # Run 802 TypeScript tests

# === Run ===
cargo run -p rlmx-cli -- serve --port 3000   # MCP server (Rust)
npx aix serve --port 3000                     # MCP server (TypeScript)
npx aix voice start                           # Voice pipeline
npx aix marketplace search --domain finance   # Search agents
npx aix mesh status                           # Personal mesh

# === Frontend ===
cd frontend && python3 -m http.server 8080    # Web dashboard at :8080
```

### With Edge Inference

```bash
mkdir -p ~/.rlmx/models
# Download GGUF model + tokenizer, then:
RLMX_EDGE_MODEL=tinyllama-1.1b-q4_k_m \
  cargo run -p rlmx-cli --features "ruvllm,metal" -- serve --port 3000
```

## CLI Commands (`npx aix`)

```bash
aix serve --port 3000              # Start MCP + WebSocket server
aix query -i "search term"         # Semantic search
aix voice start                    # Start voice pipeline
aix voice intents --text "..."     # Intent decomposition
aix agent spawn --type worker      # Spawn agent
aix marketplace search             # Search marketplace
aix mesh status                    # Personal mesh status
aix billing status                 # Subscription status
aix federation contribute          # Trigger federation cycle
aix engagement score               # Life Score (0-100)
```

## Swarm Zones

| Zone | Role | Devices |
|------|------|---------|
| A-Mobile | Primary compute | Phone |
| A-Desktop | Secondary compute | Laptop |
| B-Cloud | Burst overflow | GPU cloud |
| C-Edge | Sentinel, privacy anchor | Pi, NUC, home hub |
| D-Browser | WASM compute pool | Browser tabs |
| E-HomeHub | Long-term data store | Home server |

## Agent Types (17)

Coordinator, Researcher, Router, Experimenter, Worker, Monitor, Reviewer, Trainer, Validator, Replicator, Embedder, Analyst, VoiceCoordinator, MarketplaceManager, MeshCoordinator, FederationAgent, BillingManager

Each agent has scoped capabilities per the 17x17 permission matrix (ADR-005).

## MCP Tools (47)

28 core + 8 marketplace + 3 voice + 2 mesh + 2 federation + 4 billing. All tools enforce RBAC with 6 roles: admin, system, engineer, operator, auditor, viewer.

## Frontend Dashboard

Single-page dark-themed dashboard (`frontend/index.html`, ~2900 lines vanilla JS) with 20+ views. Works standalone with demo data or connected to the MCP server.

```bash
cd frontend && python3 -m http.server 8080
```

## Mobile App

React Native 0.84.1 + TypeScript for Android. 8 screens, 11 components, dark theme. Demo mode works offline.

```bash
cd mobile && npm install && npx react-native start
```

## Tests

| Suite | Count | Command |
|-------|-------|---------|
| Rust (19 crates) | 946+ | `cargo test --workspace` |
| TypeScript (13 packages) | 802 | `npm run test:ts` |
| **Total** | **1,748+** | `npm test` |

## Documentation

| Path | Contents |
|------|----------|
| `docs/TYPESCRIPT-MIGRATION.md` | Full TypeScript migration reference (packages, NAPI, DDD, conventions) |
| `docs/ADR/` | 29 Architecture Decision Records |
| `docs/DDD/` | 16 Domain-Driven Design documents |
| `CLAUDE.md` | AI assistant rules, build commands, strict rules, key files |
| `move-to-typescript-plan.md` | Original migration plan |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RLMX_EDGE_MODEL` | GGUF model name (without `.gguf`) | unset |
| `RLMX_MODEL_DIR` | Directory for `.gguf` + tokenizer files | `~/.rlmx/models` |
| `JAVA_HOME` | JDK path for Android builds | system default |
| `RLMX_MESH_PORT` | Personal mesh discovery port | `5353` |

## Feature Flags

| Feature | Effect |
|---------|--------|
| `ruvllm` | Enable CandleBackend for GGUF inference |
| `metal` | Apple Metal GPU acceleration (macOS) |
| `napi` | NAPI-RS Node.js native bindings |
| `wasm` | wasm-bindgen browser bindings |
| `ruvnet-phase1..5` | Ruvnet ecosystem integration phases |
