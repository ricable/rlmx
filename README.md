# RLMX — RuVix Cognition Kernel

A **voice-first cognition kernel** — an OS-kernel-inspired runtime for LLM agents, activated by voice. Provides capability-secured syscall primitives that agents call instead of accessing arbitrary APIs.

**Hybrid Rust + TypeScript** | 22 Rust crates + 20 npm packages | 82 MCP tools | 17 agent types | 6 swarm zones | 2,421+ tests

## What It Does

- **Voice-first pipeline**: On-device STT -> multi-intent decomposition -> agent swarm -> multimodal response
- **18 capability-secured syscalls** with HMAC-SHA256 tokens and hierarchical derivation
- **17 typed agent roles** with an 18x17 permission matrix
- **6-zone distributed swarm** with PBFT/Raft/Gossip consensus
- **82 MCP tools** (JSON-RPC 2.0) over HTTP + WebSocket with 6-role RBAC
- **Neural model routing** via FastGRNN (18-dim) with online learning
- **Tiered inference**: Small (0.5B) -> Medium (MLX 3-8B) -> Cloud (vLLM)
- **Edge inference** via GGUF models on CPU/Metal/CUDA
- **Browser compute** via WebAssembly (WebGPU/WASM SIMD)
- **Personal mesh** with mDNS discovery, privacy anchor pattern, cross-device sync
- **Federated learning** with differential privacy (Laplace e=1.0)
- **Agent marketplace** with 12 life domains, automated security review, 70/30 revenue split
- **Subscription billing** (6 tiers) with capability-token enforcement
- **Content-addressed artifact DAG** with SHA-256 hashing, branching, and diff
- **Coordination boards** for persistent inter-agent threaded communication
- **Per-agent budget ledger** with soft/hard limits and CAS versioning
- **External runtime bridges** for Claude Code, Codex, Cursor, and OpenAI-compatible endpoints
- **A2A protocol** (Agent-to-Agent) with JSON-RPC 2.0, agent cards, and task state machine
- **Skills marketplace** with SKILL.md format, semantic search, and 12 bundled domain skills
- **Dynamic function evolution** with lifecycle management, scoring, and feedback loops
- **Human-in-the-loop approval tiers** (Auto/Notify/Confirm/Escalate) with timeout auto-deny
- **Declarative trigger registry** for event/schedule/HTTP/channel-to-function bindings
- **Channel adapters** for Telegram, WhatsApp, Teams, and Discord

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
 rlmx-rvf      (Ed25519, witness)           @aix/billing   (6 tiers, budget)
 rlmx-napi     (NAPI bridge — 31 fns)      @aix/federation (cycles, privacy)
 rlmx-wasm     (browser kernel)             @aix/plugin    (domain plugins)
 rlmx-artifact (content-addressed DAG)      @aix/rlm       (vLLM client)
 rlmx-evolve   (function evolution)         @aix/shared    (types, events, errors)
 rlmx-channels (Telegram/WA/Teams/Discord)  @aix/core      (NAPI loader)
                                            @aix/artifact  (DAG client)
                                            @aix/a2a       (agent-to-agent)
                                            @aix/skills    (skill registry)
                                            @aix/evolve    (function evolution)
                                            @aix/triggers  (event bindings)
                                            @aix/channels  (messaging adapters)
```

## Quick Start

```bash
# === Rust ===
cargo build --workspace              # Build all 22 crates
cargo test --workspace               # Run 1,286+ Rust tests
cargo clippy --workspace -- -D warnings

# === TypeScript ===
npm install                          # Install workspace dependencies
npm run test:ts                      # Run 1,135 TypeScript tests

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

Each agent has scoped capabilities per the 18x17 permission matrix (ADR-005). The 18th permission (`ArtifactWrite`) was added in ADR-030.

## MCP Tools (82)

28 core + 8 marketplace + 3 voice + 2 mesh + 2 federation + 4 billing + 35 AgentOS integration (3 artifact + 3 board + 3 budget + 2 bridge + 3 a2a + 4 skill + 6 evolve + 3 approval + 4 trigger + 4 channel). All tools enforce RBAC with 6 roles: admin, system, engineer, operator, auditor, viewer.

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
| Rust (22 crates) | 1,286+ | `cargo test --workspace` |
| TypeScript (20 packages) | 1,135 | `npm run test:ts` |
| **Total** | **2,421+** | `npm test` |

## Documentation

| Path | Contents |
|------|----------|
| `CLAUDE.md` | AI assistant rules, conventions, strict rules (token-efficient, cross-referenced) |
| `docs/BUILD-COMMANDS.md` | Complete build, test, deploy, and CLI command reference |
| `docs/ARCHITECTURE.md` | Detailed component descriptions for all subsystems |
| `docs/WORKSPACE-STRUCTURE.md` | 19 crate tree, 13 TS packages, directories, 16 DDD contexts |
| `docs/KEY-FILES.md` | Key file locations across all crates and packages |
| `docs/FEATURE-GATES.md` | Feature gate matrix (13 features) and integration rules |
| `docs/TYPESCRIPT-MIGRATION.md` | Full TypeScript migration reference (packages, NAPI, DDD, conventions) |
| `docs/ADR/` | 39 Architecture Decision Records (ADR-001 through ADR-039) |
| `docs/DDD/` | 16 Domain-Driven Design documents (DDD-001 through DDD-016) |

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
