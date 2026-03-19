# RLMX — Recursive Language Model eXtended
## on the RuVix Cognition Kernel

### Product Requirements Document v3.0 — Definitive Architecture

| Field | Value |
|---|---|
| **Version** | 3.0.0 |
| **Date** | March 18, 2026 |
| **Status** | Architecture Proposal — Pre-Implementation |
| **Dev Environment** | MacBook Pro M3 Max, 128GB, vllm-metal |
| **Production Target** | NVIDIA H100, vLLM, colocated with data sources |
| **Foundation** | `npx ruvector` — Full RuVector/RuVix ecosystem |
| **License** | MIT / Apache-2.0 dual-license (following RuVector) |

---

## 0. Bottom Line Up Front

RLMX is a **use-case-agnostic infinite-context AI platform** where the RuVix cognition kernel is the sole execution substrate. There is no separate "RLMX Engine" — the Recursive Language Model pattern is a **scheduling policy** within RuVix. The root agent is a RuVix process. Sub-agents are child processes with scoped capability tokens. Context segments are kernel objects. Every action is a syscall. Every mutation is proof-gated.

The platform supports **two recursive reasoning paradigms** as pluggable strategies:

- **RLM** (Recursive Language Model, Zhang/Kraska/Khattab, MIT OASYS): LLM-scale models (1–30B) that recursively decompose long-context tasks through sub-LLM delegation. Best for open-ended reasoning, natural language analysis, tool use, and multi-hop retrieval across massive corpora. The model manages its own context through programmatic inspection and recursive sub-queries.

- **TRM** (Tiny Recursive Model, Jolicoeur-Martineau, Samsung SAIL Montreal): Ultra-small networks (5–7M parameters, 2 layers) that iteratively refine predictions through recursive cycles over three streams (question x, answer y, latent reasoning z). Best for structured reasoning tasks — constraint satisfaction, pattern matching, combinatorial optimization, anomaly classification. Achieves 45% on ARC-AGI-1 with 0.01% of frontier model parameters.

Domain-specific capabilities load as **plugins** through a standardized trait interface. Ericsson RAN optimization is the reference plugin. The platform is designed for **any domain** — finance, healthcare, industrial IoT, legal, cybersecurity — through the same plugin mechanism.

Development runs on a single MacBook Pro M3 Max (128GB) with vllm-metal. Production deploys vLLM on H100 GPUs colocated with domain data sources. `npx ruvector` is the universal entry point.

---

## 1. Foundational Thesis

### 1.1 Why Infinite Context Matters

Every serious AI application eventually hits the context wall. A telecom engineer needs 30 days of multi-cell telemetry to distinguish a recurring traffic pattern from a genuine anomaly. A financial analyst needs 6 months of order flow to detect a market manipulation scheme. A clinician needs a patient's complete medical history to avoid a drug interaction. Current AI systems either truncate this context (losing critical information) or attempt to fit it into ever-larger attention windows (at quadratic cost with degrading quality).

The Recursive Language Model paradigm (arXiv:2512.24601) proved this is a solved problem at the architectural level. In experiments at MIT, an RLM using GPT-5-mini outperformed GPT-5 on the OOLONG long-context benchmark by more than double the number of correct answers — while being cheaper per query. On BrowseComp+ tasks requiring 6–11 million tokens, RLM achieved 91.3% accuracy where the base model scored 0%. The critical insight: **no single model call should ever need to handle the full context**. The model recursively decomposes, delegates, and synthesizes.

The Tiny Recursive Model paradigm (arXiv:2510.04871) proved a complementary thesis: for structured reasoning tasks, you don't need large models at all. A 7M-parameter, 2-layer network achieves 45% on ARC-AGI-1 and 87% on Sudoku-Extreme — outperforming DeepSeek R1, o3-mini, and Gemini 2.5 Pro — by recursively refining its answer through multiple cycles. The cost is negligible. The latency is sub-millisecond per cycle. The deployment footprint is under 30MB.

RLMX unifies both paradigms under a single kernel.

### 1.2 Why RuVix Is the Only Kernel

RuVix is a cognition kernel designed for agentic workloads. Where Linux thinks in files, processes, and POSIX syscalls, RuVix thinks in **vectors, graphs, proofs, and capabilities**. Every mutation is proof-gated — no cryptographic proof, no state change. Every resource access goes through unforgeable capability tokens. 20+ crates, 1,200+ tests, 12 syscalls, full deterministic replay.

The mapping from RLM/TRM primitives to RuVix syscalls is direct:

| RLM/TRM Primitive | RuVix Equivalent | Notes |
|---|---|---|
| Store context as variable | **Region Memory** (kernel object) | Segments are first-class kernel objects with capability tokens |
| Semantic search over context | **Vector syscall** → ruvector-core HNSW | <0.5ms p50, three-tier automatic quantization |
| Spawn sub-LLM | **Fork process** with scoped capabilities | Child inherits read-only memory view, gets own region |
| Tool delegation | **Capability Manager** grants tool tokens | Sub-process can only use tools its capability token permits |
| Recursive sub-query | **Queue IPC** between parent/child processes | Message-passing with proof-of-delivery |
| Proof-gated mutation | **Proof Engine** validates before state change | Ed25519 + ML-DSA-65 witness chain |
| Self-learning | **SONA** adaptation in <100μs | Pattern bank, micro-LoRA, EWC++ |
| Coherence checking | **Cognitum Gate** blocks contradictory mutations | Sheaf-theoretic consistency verification |
| TRM latent refinement | **Region Memory** cycles over (x, y, z) tensors | Zero-copy iteration via RuVix memory mapping |
| Adaptive halting | **Circadian Controller** + coherence threshold | Stop when improvement < ε or coherence peaks |

There is no "RLMX Engine" sitting on top of RuVix. The recursive reasoning loop **is** a RuVix scheduling policy. This eliminates an entire architectural layer, reduces latency, and enables kernel-level optimizations (eBPF hot-path acceleration, direct memory mapping) that would be impossible through an intermediary.

### 1.3 Two Recursive Strategies, One Kernel

```
┌─────────────────────────────────────────────────────────────────┐
│                    RuVix Cognition Kernel                         │
│                                                                   │
│   Capability Manager │ Queue IPC │ Coherence-Aware Scheduler     │
│   Region Memory │ Proof Engine │ Vector/Graph Kernel Objects     │
│                                                                   │
│   ┌─────────────────────────┐  ┌──────────────────────────────┐ │
│   │  RLM Strategy            │  │  TRM Strategy                 │ │
│   │  (Scheduling Policy)     │  │  (Scheduling Policy)          │ │
│   │                          │  │                               │ │
│   │  Root Process:           │  │  Single Process:              │ │
│   │  • 1-30B LLM via vLLM   │  │  • 5-7M TRM via ONNX/Candle  │ │
│   │  • Decomposes tasks      │  │  • 3 streams: x, y, z        │ │
│   │  • Spawns sub-processes  │  │  • K refinement cycles        │ │
│   │  • Tools via capabilities│  │  • Adaptive halting           │ │
│   │  • Natural language I/O  │  │  • Structured I/O only        │ │
│   │                          │  │                               │ │
│   │  Best for:               │  │  Best for:                    │ │
│   │  • Open-ended reasoning  │  │  • Constraint satisfaction    │ │
│   │  • Multi-hop retrieval   │  │  • Pattern classification     │ │
│   │  • Tool-heavy workflows  │  │  • Anomaly detection          │ │
│   │  • Document analysis     │  │  • Combinatorial optimization │ │
│   │  • Natural language Q&A  │  │  • Grid/graph puzzles         │ │
│   └─────────────────────────┘  └──────────────────────────────┘ │
│                                                                   │
│   Both strategies share:                                          │
│   • HNSW context memory (ruvector-core)                          │
│   • Self-learning (SONA + DAG optimizer)                         │
│   • Proof-gated mutations (cognitum-gate)                        │
│   • Graph analysis (ruvector-gnn + mincut)                       │
│   • RVF container packaging                                      │
│   • Plugin interface for domain specifics                        │
└─────────────────────────────────────────────────────────────────┘
```

A plugin can declare which strategy it prefers for each sub-task. The Ericsson RAN plugin, for example, might use **RLM** for root-cause analysis ("Why did Cell-47 throughput degrade?") and **TRM** for anomaly classification ("Is this PM counter pattern normal or anomalous?" — a structured grid-like task where TRM's latent refinement excels with negligible compute).

---

## 2. Full RuVector Crate Integration

### 2.1 Kernel Layer (RuVix)

The RuVix cognition kernel integrates 10 core RuVector crates into a unified execution environment with 12 syscalls:

| RuVix Syscall | Underlying Crates | Function |
|---|---|---|
| `vec_insert` | ruvector-core, rvf-quant | Insert embedding into HNSW with metadata, auto-tier |
| `vec_search` | ruvector-core, ruvector-filter, ruvector-router-core | k-NN search with boolean filters, routed to optimal strategy |
| `vec_delete` | ruvector-core, ruvector-delta-core | Remove segment, track delta for federation sync |
| `graph_query` | ruvector-gnn, ruvector-graph | Cypher-like graph queries over entity relationships |
| `graph_cut` | ruvector-mincut | Dynamic min-cut: find critical connections, community boundaries |
| `graph_diffuse` | ruvector-solver | Heat kernel diffusion for knowledge propagation |
| `process_fork` | RuVix Capability Manager | Spawn child process with scoped capability token |
| `process_send` | RuVix Queue IPC | Send message to child/parent process |
| `process_recv` | RuVix Queue IPC | Receive message (blocking or async) |
| `state_mutate` | cognitum-gate-kernel, rvf-crypto | Proof-gated state change with witness generation |
| `attention_select` | ruvector-attention (39 mechanisms) | Auto-select optimal attention for current operation |
| `halt_check` | ruvector-nervous-system (circadian + WTA) | Adaptive halting: should this process continue or return? |

### 2.2 Cognitive Layer

| Crate | Integration | Performance |
|---|---|---|
| **ruvector-attention** | 39 attention mechanisms: Flash, Topological, Causal Cone, Critical Path, MinCut Gated, Cross, Multi-Head, Sliding Window, Poincaré, Graph Attention, etc. Auto-selected per operation via `attention_select` syscall | Benchmark via `npx ruvector attention benchmark` |
| **ruvector-gnn** | Graph Neural Networks model entity relationships discovered in context. Cell topology graphs, document citation networks, financial entity graphs, patient treatment pathways | ruvector-graph Cypher queries |
| **ruvector-mincut** | Dynamic min-cut (arXiv:2512.13105 breakthrough) identifies critical connections. In RAN: which cell relationships are most important? In finance: which counterparty connections are systemic? | Deterministic exact, subpolynomial updates, 448+ tests |
| **ruvector-solver** | PageRank, graph connectivity, AI attention computation at scale. Computes influence propagation across context graphs | AVX2 SIMD SpMV, fused residual kernels, arena allocator |
| **ruvector-dag** | Self-learning query optimizer. Watches execution, learns optimal strategy per query pattern. 7 attention mechanisms, SONA integration | 50-80% latency reduction after ~100 queries |
| **ruvector-nervous-system** | BTSP one-shot learning (no training loop), HDC 10K-bit hypervectors (<50ns bind, <100ns similarity), WTA competition (<1μs), oscillatory router (40Hz gamma), circadian controller (24h compute/learn/consolidate cycles), global workspace (4-7 item attention focus) | All sub-microsecond latencies |
| **sona** | Self-Optimizing Neural Architecture. Micro-LoRA adaptation (<100μs), EWC++ catastrophic forgetting prevention, pattern bank (successful query→action→result patterns indexed in HNSW) | Measurable improvement after 100 interactions |

### 2.3 RVF Container Layer

| Crate | Role | Detail |
|---|---|---|
| **rvf-types** | 24+ segment types | VEC, INDEX, MANIFEST, QUANT, WITNESS, CRYPTO, KERNEL, EBPF, WASM, COW_MAP, MEMBERSHIP, DELTA, TRANSFER_PRIOR, POLICY_KERNEL, COST_CURVE, OVERLAY (LoRA), GRAPH, SKETCH, and more |
| **rvf-wire** | Binary format | Zero-copy reads, crash-safe atomic writes without WAL |
| **rvf-manifest** | Root manifest | Single page read, cold boot <5ms |
| **rvf-quant** | Three-tier quantization | Hot fp16 → warm PQ → cold binary, automatic |
| **rvf-index** | HNSW serialization | Layer A/B/C, first query at 70% recall before full load |
| **rvf-crypto** | Cryptography | Ed25519 signing + ML-DSA-65 post-quantum |
| **rvf-runtime** | Execution runtime | Opens, queries, mutates RVF files |
| **rvf-kernel** | Linux microkernel | Self-booting: drop on VM → service in <125ms |
| **rvf-ebpf** | Kernel acceleration | Hot vectors in kernel data path, bypass userspace |
| **rvf-federation** | Multi-node sync | Distributed RLMX instances share context |
| **rvf-launch** | Orchestration | Start RVF on appropriate compute tier |
| **rvf-server** | HTTP serving | Production endpoint for RVF queries |
| **rvf-import** | Format conversion | GGUF, safetensors, CSV, JSON → RVF segments |
| **rvf-cli** | 17 commands | inspect, query, seal, branch, merge, diff, etc. |
| **rvf-wasm** | Browser runtime | 5.5KB, no backend, same queries as native |
| **rvf-solver-wasm** | Browser solver | Client-side graph analysis |
| **rvf-node** | N-API bindings | TypeScript SDK with lineage, kernel/eBPF, inspection |

### 2.4 Agent & Application Adapters

| Adapter | Integration |
|---|---|
| **claude-flow** | Ruflo swarm orchestration: hierarchical multi-agent coordination, blackboard IPC at `.swarm/memory.db`, 215 MCP tools |
| **agentdb** | Agent state persistence in SQLite, cross-agent knowledge transfer |
| **agentic-flow** | Hot-swap between inference providers (vLLM, Ollama, OpenAI, Anthropic) |
| **ospipe** | Screen/activity capture → HNSW indexing with PII safety gate, 61μs p50 query, 11.8KB WASM micro bundle |
| **rvlite** | Edge-optimized embedded DB for IoT/constrained deployment |
| **sona** | Self-learning hooks for Claude Code integration |

### 2.5 npm Packages (`npx ruvector` Entry Point)

| Package | Purpose |
|---|---|
| `npx ruvector` | Master CLI: vector DB, MCP server, attention, hooks, graph, DAG |
| `npx ruvector mcp start` | Start MCP server |
| `npx ruvector attention list` | List 39 attention mechanisms |
| `npx ruvector attention benchmark` | Performance comparison |
| `npx @ruvector/cli hooks init` | Install self-learning hooks |
| `@ruvector/rvf` | TypeScript RVF SDK |
| `@ruvector/rvf-node` | N-API native bindings |
| `@ruvector/rvf-wasm` | 5.5KB browser WASM runtime |
| `@ruvector/rvf-mcp-server` | MCP server for RVF operations |
| `@ruvector/ruvllm-wasm` | Browser LLM inference |
| `@ruvector/attention` | Attention module CLI + library |

### 2.6 PostgreSQL (Production Memory)

```sql
CREATE EXTENSION ruvector;

-- Context segments table
CREATE TABLE rlmx_segments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  embedding ruvector(384),
  content TEXT,
  source TEXT,
  plugin TEXT,
  segment_type TEXT,
  timestamp TIMESTAMPTZ NOT NULL,
  metadata JSONB,
  tier TEXT DEFAULT 'hot'  -- hot/warm/cold, managed by rvf-quant
);

-- HNSW index with cosine distance
CREATE INDEX ON rlmx_segments USING ruhnsw (embedding ruvector_cosine_ops);

-- Semantic search: <0.5ms p50 at 100K segments
SELECT content, embedding <=> query_embedding AS distance
FROM rlmx_segments
WHERE plugin = 'ericsson-ran'
  AND timestamp > NOW() - INTERVAL '7 days'
ORDER BY distance
LIMIT 20;

-- Hyperbolic distance for hierarchical context (cell topology)
SELECT ruvector_poincare_distance(a.embedding, b.embedding, -1.0)
FROM rlmx_segments a, rlmx_segments b
WHERE a.segment_type = 'cell_config' AND b.segment_type = 'cell_config';

-- Graph diffusion for knowledge propagation
SELECT ruvector_graph_diffusion(adjacency_json, signal);

-- Self-healing
SELECT ruvector_auto_repair('rlmx_segments_embedding_idx');
```

---

## 3. Architecture Deep Dive

### 3.1 RLM Strategy on RuVix

When a query arrives that requires open-ended reasoning over large context, RuVix schedules the **RLM strategy**:

```
User Query: "Why did Cell-47 throughput degrade last Thursday evening,
             and is it getting worse over time?"

RuVix Scheduler → RLM Strategy (open-ended, multi-hop, temporal)

┌─────────────────────────────────────────────────────────────────┐
│  RuVix Process: Root Agent                                       │
│  Model: vLLM-served Qwen3-8B (M3 Max dev / H100 prod)          │
│  Context window: 4-8K tokens (deliberately small)                │
│  Capability token: [vec_search, graph_query, process_fork,       │
│                     state_mutate, attention_select]               │
│                                                                   │
│  Syscall 1: vec_search(                                          │
│    query="Cell-47 throughput degradation Thursday evening",       │
│    k=20, filters={time_range: "7d", plugin: "ericsson-ran"}     │
│  )                                                                │
│  → ruvector-router selects Hybrid strategy                       │
│  → ruvector-filter applies metadata constraints                  │
│  → 20 segments returned in <0.5ms                                │
│                                                                   │
│  Model reasons: recurring pattern, need historical comparison    │
│                                                                   │
│  Syscall 2: process_fork(                                        │
│    capabilities=[vec_search],  // read-only memory access        │
│    memory_scope="Cell-47 AND Thursday AND time_range:30d",       │
│    task="Analyze 30-day Thursday evening throughput trend"        │
│  )                                                                │
│  → Child process spawned with scoped capability token            │
│  → Child can ONLY search Cell-47 Thursday data                   │
│                                                                   │
│  Syscall 3: process_fork(                                        │
│    capabilities=[vec_search, graph_query],                       │
│    memory_scope="Cell-45,46,47,48 AND time_range:7d",           │
│    task="Analyze neighbor cell interference correlation"          │
│  )                                                                │
│  → Another child, different scope, additional graph capability   │
│                                                                   │
│  Syscall 4: process_recv(children, timeout=2s)                   │
│  → Parallel results from both children                           │
│                                                                   │
│  Child 1 result: "Throughput degradation worsening weekly:       │
│    -25% (4wk ago) → -28% (2wk ago) → -30% (now)"               │
│  Child 2 result: "Cell-45 PRB utilization climbing Thursday      │
│    evenings (new office building), causing interference"          │
│                                                                   │
│  Model synthesizes root cause + recommendation                   │
│                                                                   │
│  Syscall 5: state_mutate(                                        │
│    action="OptimizeParameter",                                   │
│    target="Cell-45",                                             │
│    parameter="cellIndividualOffsetEUtran",                       │
│    value=-1,                                                     │
│    proof={reasoning_chain, evidence_segments, confidence: 0.87}  │
│  )                                                                │
│  → cognitum-gate validates: within safety bounds ✓               │
│  → rvf-crypto generates witness: Ed25519 + ML-DSA-65            │
│  → State change committed with full audit trail                  │
│                                                                   │
│  Total: ~450ms (H100) / ~1.5s (M3 Max)                          │
│  Effective context: 30 days × 50 cells × 100 KPIs               │
│  Actual tokens in attention: ~6K per agent                       │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 TRM Strategy on RuVix

When a task requires structured reasoning — pattern classification, constraint satisfaction, anomaly detection — RuVix schedules the **TRM strategy**:

```
Trigger: New PM counter snapshot for Cell-47 arrived.
         Classify: normal / degraded / anomalous?

RuVix Scheduler → TRM Strategy (structured classification, low-latency)

┌─────────────────────────────────────────────────────────────────┐
│  RuVix Process: TRM Classifier                                   │
│  Model: 7M-param TRM (2-layer, embedded in RLMX binary)        │
│  No vLLM needed — runs directly on CPU in <10ms                 │
│                                                                   │
│  Input streams:                                                   │
│  x (question): Current PM snapshot (50 counters, normalized)     │
│  y (initial answer): [normal=0.33, degraded=0.33, anomalous=0.33]│
│  z (latent reasoning): Initialized from SONA pattern bank        │
│                                                                   │
│  Cycle 1 (0.5ms):                                                │
│    z ← refine(x, y, z)  // Latent reasoning update              │
│    y ← refine(x, y, z)  // Answer refinement                    │
│    halt_check → continue (confidence < threshold)                │
│                                                                   │
│  Cycle 2 (0.5ms):                                                │
│    z ← refine(x, y, z)  // z encodes temporal context from SONA │
│    y ← refine(x, y, z)  // y sharpening                         │
│    halt_check → continue                                         │
│                                                                   │
│  ... cycles 3-8 ...                                               │
│                                                                   │
│  Cycle 9 (0.5ms):                                                │
│    y = [normal=0.12, degraded=0.85, anomalous=0.03]              │
│    halt_check → HALT (confidence 0.85 > threshold 0.80)          │
│                                                                   │
│  Result: degraded (0.85 confidence)                              │
│  → Triggers RLM strategy for root-cause analysis                 │
│  → SONA records pattern for future classification                │
│                                                                   │
│  Total: ~4.5ms, ~30MB memory, no GPU required                   │
│  Can run on edge, browser (WASM), IoT, anywhere                  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.3 Hybrid Strategy: TRM Triage → RLM Deep Analysis

The real power emerges when both strategies work together:

```
Streaming PM Data (every 15 min, 50 cells)
    │
    ▼
┌────────────────────────────────────────┐
│  TRM Triage (per cell, <5ms each)      │
│  Classify: normal / degraded / anomaly │
│  250 classifications in ~250ms total    │
│  Runs on CPU, no GPU needed            │
└──────────────┬─────────────────────────┘
               │
    ┌──────────┼──────────────┐
    ▼          ▼              ▼
 45 normal  4 degraded     1 anomalous
 (no action) (log + watch)  (escalate)
                              │
                              ▼
               ┌──────────────────────────┐
               │  RLM Deep Analysis       │
               │  Full recursive decomp.  │
               │  30-day context retrieval │
               │  Multi-cell correlation   │
               │  Root cause identification│
               │  Optimization action      │
               │  ~500ms on H100           │
               └──────────────────────────┘
```

This pattern — cheap TRM triage followed by expensive RLM deep analysis only when needed — reduces total GPU cost by **90%+** compared to running RLM on every PM snapshot.

### 3.4 Development Environment Detail

**MacBook Pro M3 Max, 128GB unified memory:**

```bash
# 1. Install vllm-metal (official vLLM Apple Silicon plugin)
curl -fsSL https://raw.githubusercontent.com/vllm-project/vllm-metal/main/install.sh | bash
source ~/.venv-vllm-metal/bin/activate

# 2. Serve model for RLM strategy development
vllm serve mlx-community/Qwen3-8B-4bit \
  --host 127.0.0.1 --port 8000 \
  --api-key "${VLLM_API_KEY}"
# ~4.5GB model, leaves ~120GB for HNSW + KV cache + dev tools

# 3. Install RuVector ecosystem
npm install ruvector
npx ruvector mcp start                    # MCP server on :3000
npx @ruvector/cli hooks init              # Self-learning hooks

# 4. Start ruvector-postgres for persistent memory
docker run -d --name ruvector-pg \
  -e POSTGRES_PASSWORD=secret \
  -p 5432:5432 \
  ruvnet/ruvector-postgres:latest

# 5. Rust workspace for RLMX development
cargo init rlmx-workspace --name rlmx
cd rlmx-workspace

# Core kernel integration
cargo add ruvector-core ruvector-filter ruvector-cluster
cargo add ruvector-delta-core ruvector-router-core
cargo add cognitum-gate-kernel

# Cognitive layer
cargo add ruvector-attention ruvector-gnn ruvector-mincut
cargo add ruvector-solver ruvector-dag ruvector-nervous-system

# RVF containers
cargo add rvf-runtime rvf-crypto rvf-wasm rvf-index rvf-quant
cargo add rvf-import rvf-manifest rvf-types rvf-wire

# TRM inference (lightweight, no GPU needed)
cargo add candle-core candle-nn candle-transformers  # For TRM model execution
cargo add ort  # ONNX Runtime for TRM alternative backend

# 6. Memory budget on M3 Max 128GB:
#    vLLM + Qwen3-8B Q4:    ~4.5GB + ~8GB KV cache = ~12.5GB
#    ruvector-postgres:       ~2GB base + ~4GB HNSW (500K segments)
#    TRM models (all):        ~100MB (7M params × 4 bytes × ~3 models)
#    Rust dev toolchain:      ~2GB
#    OS + apps:               ~8GB
#    REMAINING:               ~100GB free for experimentation
```

### 3.5 Production Environment Detail

```bash
# H100 server colocated with Ericsson PM solution

# 1. vLLM on H100 (80GB HBM3)
pip install vllm
vllm serve Qwen/Qwen3-30B-A3B-FP8 \
  --host 0.0.0.0 --port 8000 \
  --tensor-parallel-size 1 \
  --max-model-len 32768 \
  --enable-prefix-caching \
  --api-key "${VLLM_API_KEY}"
# Throughput: 300-500 tok/s, batch inference for parallel sub-agents

# 2. ruvector-postgres (separate high-memory instance)
docker run -d --name ruvector-pg \
  -e POSTGRES_PASSWORD="${PG_PASSWORD}" \
  -e POSTGRES_DB=rlmx \
  -p 5432:5432 \
  -v /data/ruvector:/var/lib/postgresql/data \
  --shm-size=4g \
  ruvnet/ruvector-postgres:latest

# 3. RLMX binary (Rust native, connects to vLLM + postgres)
./rlmx serve \
  --vllm-endpoint http://localhost:8000 \
  --postgres-url postgres://rlmx:${PG_PASSWORD}@localhost:5432/rlmx \
  --plugins ./plugins/ericsson-ran.so \
  --mcp-port 8080 \
  --witness-chain enabled \
  --sona-enabled true

# 4. Data source connection (Ericsson PM)
# Option A: Direct file access to PM XML exports
./rlmx ingest \
  --plugin ericsson-ran \
  --adapter pm-xml \
  --source /data/ericsson/pm/ \
  --watch true  # Continuous monitoring for new files

# Option B: API access to PM solution
./rlmx ingest \
  --plugin ericsson-ran \
  --adapter pm-api \
  --endpoint https://enm.operator.com/pm/v1 \
  --auth-token "${ENM_TOKEN}" \
  --poll-interval 60s
```

---

## 4. Plugin Architecture

### 4.1 Plugin Trait Definition

```rust
/// Every domain plugin implements this trait.
/// Plugins are loaded as:
///   - Compiled into the RLMX binary (built-in)
///   - Dynamic shared library (.so / .dylib)
///   - WASM module (sandboxed, portable)
///   - RVF container (self-contained knowledge + code)
pub trait DomainPlugin: Send + Sync + 'static {
    // ── Identity ──────────────────────────────────────────────
    fn name(&self) -> &str;
    fn version(&self) -> semver::Version;
    fn description(&self) -> &str;

    // ── Data Ingestion ────────────────────────────────────────
    /// Returns adapters that convert domain-specific formats
    /// into normalized ContextSegments for HNSW indexing.
    fn ingest_adapters(&self) -> Vec<Box<dyn IngestAdapter>>;

    // ── Reasoning Strategy Preferences ────────────────────────
    /// For each task type, declare preferred strategy (RLM, TRM, or Auto).
    /// Auto lets RuVix scheduler decide based on task structure.
    fn strategy_preferences(&self) -> Vec<StrategyPreference>;

    // ── Action Grammar Extensions ─────────────────────────────
    /// Domain-specific actions added to the constrained grammar.
    /// These become available as RuVix syscall extensions.
    fn action_extensions(&self) -> Vec<ActionDefinition>;

    // ── Embedding Configuration ───────────────────────────────
    /// Optional domain-specific embedding model.
    /// Default: all-MiniLM-L6-v2 (384 dimensions).
    fn embedding_config(&self) -> Option<EmbeddingConfig> { None }

    /// Optional domain-specific distance metric.
    /// Default: cosine. Options: L2, inner product, Manhattan,
    /// Wasserstein, Poincaré (hyperbolic), product manifold.
    fn distance_metric(&self) -> Option<DistanceMetric> { None }

    // ── TRM Models ────────────────────────────────────────────
    /// Optional TRM models for structured reasoning sub-tasks.
    /// Plugin provides pre-trained .onnx or .gguf TRM weights.
    fn trm_models(&self) -> Vec<TrmModelConfig> { vec![] }

    // ── System Prompt Extensions ──────────────────────────────
    /// Added to the root agent's system prompt when this plugin is active.
    fn system_prompt_extension(&self) -> String;

    // ── Safety Constraints ────────────────────────────────────
    /// Parameter bounds, KPI guards, rollback triggers, escalation rules.
    /// Enforced by cognitum-gate-kernel during state_mutate syscalls.
    fn safety_constraints(&self) -> Vec<SafetyConstraint>;

    // ── Evaluation ────────────────────────────────────────────
    /// Domain-specific quality metrics for measuring RLMX effectiveness.
    fn evaluator(&self) -> Option<Box<dyn DomainEvaluator>> { None }

    // ── RVF Packaging ─────────────────────────────────────────
    /// Additional RVF segment types this plugin defines.
    fn rvf_segment_types(&self) -> Vec<SegmentTypeDefinition> { vec![] }

    // ── Knowledge Base ────────────────────────────────────────
    /// Optional RVF container with pre-indexed domain knowledge.
    /// Loaded into HNSW at plugin activation.
    fn knowledge_base(&self) -> Option<PathBuf> { None }

    // ── Graph Schema ──────────────────────────────────────────
    /// Entity types and relationship types for ruvector-gnn.
    /// Defines the domain's knowledge graph structure.
    fn graph_schema(&self) -> Option<GraphSchema> { None }
}

pub struct StrategyPreference {
    pub task_pattern: String,       // Regex or keyword match
    pub strategy: Strategy,         // RLM, TRM, or Auto
    pub trm_model: Option<String>,  // Which TRM model if TRM
    pub rationale: String,          // Why this strategy
}

pub enum Strategy {
    Rlm,                // Use LLM recursive decomposition
    Trm(String),        // Use named TRM model
    Auto,               // Let RuVix scheduler decide
    Hybrid {            // TRM triage, RLM on escalation
        triage: String,    // TRM model name for initial classification
        threshold: f32,    // Escalate to RLM if confidence < threshold
    },
}
```

### 4.2 Ericsson RAN Plugin (Reference Implementation)

```rust
pub struct EricssonRanPlugin {
    feature_db: EricssonFeatureDb,  // 588 features, 6000+ params, 4000+ counters
    mo_spec: ManagedObjectSpec,     // 5gmo.csv — parameter placement validation
    trm_anomaly: TrmModel,         // 7M-param TRM for PM anomaly classification
    trm_pattern: TrmModel,         // 5M-param TRM for traffic pattern recognition
}

impl DomainPlugin for EricssonRanPlugin {
    fn name(&self) -> &str { "ericsson-ran" }
    fn version(&self) -> semver::Version { semver::Version::new(1, 0, 0) }
    fn description(&self) -> &str {
        "Ericsson LTE/NR RAN optimization: 588 features, PM/FM/CM ingestion, \
         E2SM-KPM/RC integration, parameter optimization with safety bounds"
    }

    fn ingest_adapters(&self) -> Vec<Box<dyn IngestAdapter>> {
        vec![
            Box::new(PmXmlAdapter::new()),      // PM counter XML (15-min granularity)
            Box::new(PmApiAdapter::new()),       // PM REST API
            Box::new(FmAlarmAdapter::new()),     // Fault Management alarms (real-time)
            Box::new(CmExportAdapter::new()),    // Configuration Management exports
            Box::new(E2smKpmAdapter::new()),     // O-RAN E2SM-KPM reports
            Box::new(CsvCounterAdapter::new()),  // Generic CSV PM counter format
        ]
    }

    fn strategy_preferences(&self) -> Vec<StrategyPreference> {
        vec![
            StrategyPreference {
                task_pattern: "classify|anomaly|normal|degraded".into(),
                strategy: Strategy::Trm("ericsson-pm-anomaly".into()),
                trm_model: Some("ericsson-pm-anomaly".into()),
                rationale: "PM snapshot classification is structured — \
                           TRM excels at pattern recognition with <5ms latency".into(),
            },
            StrategyPreference {
                task_pattern: "traffic.*pattern|demand.*predict".into(),
                strategy: Strategy::Trm("ericsson-traffic-pattern".into()),
                trm_model: Some("ericsson-traffic-pattern".into()),
                rationale: "Traffic pattern recognition is grid-like — \
                           TRM's latent refinement ideal for temporal grids".into(),
            },
            StrategyPreference {
                task_pattern: "why|root.*cause|explain|analyze|optimize".into(),
                strategy: Strategy::Rlm,
                trm_model: None,
                rationale: "Open-ended analysis requires LLM reasoning, \
                           multi-hop retrieval, natural language output".into(),
            },
            StrategyPreference {
                task_pattern: "monitor|watch|triage".into(),
                strategy: Strategy::Hybrid {
                    triage: "ericsson-pm-anomaly".into(),
                    threshold: 0.7,
                },
                rationale: "TRM classifies every PM snapshot cheaply. \
                           Only escalate to RLM when TRM is uncertain".into(),
            },
        ]
    }

    fn trm_models(&self) -> Vec<TrmModelConfig> {
        vec![
            TrmModelConfig {
                name: "ericsson-pm-anomaly".into(),
                path: "models/ericsson-pm-anomaly-7m.onnx".into(),
                params: 7_000_000,
                layers: 2,
                input_dim: 50,     // 50 normalized PM counters
                output_classes: 3, // normal, degraded, anomalous
                max_cycles: 16,
                halt_threshold: 0.80,
            },
            TrmModelConfig {
                name: "ericsson-traffic-pattern".into(),
                path: "models/ericsson-traffic-pattern-5m.onnx".into(),
                params: 5_000_000,
                layers: 2,
                input_dim: 96,     // 96 time slots (24h × 4 per hour)
                output_classes: 8, // 8 traffic pattern archetypes
                max_cycles: 12,
                halt_threshold: 0.75,
            },
        ]
    }

    fn action_extensions(&self) -> Vec<ActionDefinition> {
        vec![
            ActionDefinition::new("OptimizeParameter")
                .param("mo_class", ParamType::String)      // EUtranCellFDD, NRCellDU
                .param("parameter", ParamType::String)      // cellIndividualOffsetEUtran
                .param("value", ParamType::Float)
                .param("target_cells", ParamType::StringArray)
                .param("confidence_threshold", ParamType::Float)
                .description("Generate parameter change with safety validation"),
            ActionDefinition::new("FeatureLookup")
                .param("feature_acronym", ParamType::String) // IFLB, DUAC, MCPC
                .description("Query 588 features, 6000+ params from Ericsson feature DB"),
            ActionDefinition::new("ParameterValidate")
                .param("mo_class", ParamType::String)
                .param("parameter", ParamType::String)
                .param("value", ParamType::Float)
                .description("Validate parameter value against 5gmo.csv specification"),
        ]
    }

    fn safety_constraints(&self) -> Vec<SafetyConstraint> {
        vec![
            SafetyConstraint::ParameterBound {
                name: "cellIndividualOffsetEUtran".into(),
                min: -24.0, max: 24.0, max_delta_per_cycle: 2.0,
            },
            SafetyConstraint::ParameterBound {
                name: "qRxLevMin".into(),
                min: -140.0, max: -44.0, max_delta_per_cycle: 4.0,
            },
            SafetyConstraint::KpiGuard {
                metric: "hoSuccessRate".into(),
                min_value: 0.95,
                rollback_if_below: true,
                observation_window: Duration::from_secs(900), // 15 min
            },
            SafetyConstraint::KpiGuard {
                metric: "rrcConnectionSuccessRate".into(),
                min_value: 0.98,
                rollback_if_below: true,
                observation_window: Duration::from_secs(900),
            },
            SafetyConstraint::HumanEscalation {
                condition: "confidence < 0.6 OR affects_cells > 10".into(),
                message: "Low confidence or wide blast radius — requires approval".into(),
            },
            SafetyConstraint::RateLimit {
                max_changes_per_hour: 20,
                max_cells_per_change: 5,
            },
        ]
    }

    fn graph_schema(&self) -> Option<GraphSchema> {
        Some(GraphSchema {
            node_types: vec![
                NodeType::new("Cell", vec!["cellId", "technology", "band", "sector"]),
                NodeType::new("Site", vec!["siteId", "location", "type"]),
                NodeType::new("Feature", vec!["acronym", "faj", "status"]),
            ],
            edge_types: vec![
                EdgeType::new("NEIGHBORS", "Cell", "Cell", vec!["distance", "handoverRate"]),
                EdgeType::new("HOSTED_ON", "Cell", "Site", vec![]),
                EdgeType::new("ACTIVATES", "Cell", "Feature", vec!["activationDate"]),
            ],
        })
    }

    fn knowledge_base(&self) -> Option<PathBuf> {
        Some(PathBuf::from("plugins/ericsson-ran-knowledge.rvf"))
        // Contains: 588 feature embeddings, parameter specs, optimization patterns
    }

    fn system_prompt_extension(&self) -> String {
        r#"You are an Ericsson RAN optimization expert. You have access to:
- 588 Ericsson features with 6000+ parameters via FeatureLookup
- PM counters at 15-minute granularity streamed into HNSW memory
- Cell topology graph with neighbor relationships
- Parameter validation against 5gmo.csv specification
- Safety bounds on all parameter changes

Always validate parameters before recommending changes.
Always check KPI guards before committing mutations.
Always provide confidence scores with recommendations.
Use TRM anomaly classification for initial screening.
Use full RLM analysis only for degraded/anomalous cells."#.into()
    }
}
```

### 4.3 Plugin Template for New Domains

```rust
/// Template: create a new domain plugin in 5 minutes
/// Copy this, fill in your domain specifics.

pub struct MyDomainPlugin;

impl DomainPlugin for MyDomainPlugin {
    fn name(&self) -> &str { "my-domain" }
    fn version(&self) -> semver::Version { semver::Version::new(0, 1, 0) }
    fn description(&self) -> &str { "My domain plugin description" }

    fn ingest_adapters(&self) -> Vec<Box<dyn IngestAdapter>> {
        vec![
            Box::new(JsonLinesAdapter::new()),  // Generic JSON-lines ingest
            // Add your domain-specific adapters here
        ]
    }

    fn strategy_preferences(&self) -> Vec<StrategyPreference> {
        vec![
            StrategyPreference {
                task_pattern: ".*".into(),
                strategy: Strategy::Auto,  // Let RuVix decide
                trm_model: None,
                rationale: "Default: auto-select based on task structure".into(),
            },
        ]
    }

    fn action_extensions(&self) -> Vec<ActionDefinition> { vec![] }
    fn system_prompt_extension(&self) -> String { String::new() }
    fn safety_constraints(&self) -> Vec<SafetyConstraint> { vec![] }
}
```

---

## 5. Self-Learning Architecture

### 5.1 Three Learning Loops

RLMX implements three concurrent learning loops, all managed by RuVix syscalls:

**Loop 1: SONA Micro-Adaptation (<100μs per query)**
Every query→action→result triple is evaluated. Successful patterns get micro-LoRA delta updates applied to the inference pathway. EWC++ prevents catastrophic forgetting. The pattern bank (HNSW-indexed) grows with every interaction. After ~100 queries, retrieval strategy selection improves measurably.

**Loop 2: DAG Query Optimization (continuous)**
The ruvector-dag crate watches every query execution path. It learns which of its 7 attention mechanisms (Topological, Causal Cone, Critical Path, MinCut Gated, etc.) performs best for which query pattern. Rising MinCut "tension" triggers automatic strategy switching. 50-80% latency reduction over time, with no code changes.

**Loop 3: Nervous System Consolidation (circadian)**
The ruvector-nervous-system circadian controller schedules three phases:
- **Compute phase** (high activity hours): Full inference, maximum responsiveness
- **Learn phase** (medium activity): SONA patterns consolidated, BTSP one-shot learning from rare events
- **Consolidate phase** (low activity): HNSW compaction, cold-tier compression, graph pruning, witness chain archival

### 5.2 Cross-Plugin Learning

When multiple plugins are active, patterns learned in one domain can transfer to others through the shared SONA pattern bank. Example: a traffic pattern recognition capability learned by the Ericsson RAN plugin could benefit a smart city traffic management plugin, since both deal with temporal load patterns. Transfer happens automatically via hyperdimensional computing (HDC) similarity in the nervous system's global workspace.

---

## 6. Security, Auditability, and Compliance

### 6.1 Proof-Gated Mutation Flow

```
Proposed Action (from RLM or TRM)
    │
    ▼
┌─────────────────────────────────────────┐
│  Plugin Safety Constraints Check         │
│  • Parameter bounds                      │
│  • KPI guards                            │
│  • Rate limits                           │
│  • Human escalation rules                │
│  PASS / REJECT with reason               │
└─────────────┬───────────────────────────┘
              │ PASS
              ▼
┌─────────────────────────────────────────┐
│  Cognitum Gate Coherence Check           │
│  • Does this contradict existing state?  │
│  • Is the reasoning chain consistent?    │
│  • Does the confidence meet threshold?   │
│  • Sheaf-theoretic residual < limit?     │
│  PASS / REJECT with coherence score      │
└─────────────┬───────────────────────────┘
              │ PASS
              ▼
┌─────────────────────────────────────────┐
│  Witness Generation (rvf-crypto)         │
│  • Ed25519 signature (fast)              │
│  • ML-DSA-65 post-quantum signature      │
│  • Timestamp (NTP-verified)              │
│  • Agent ID + action hash                │
│  • Full reasoning chain hash             │
│  • Evidence segment references           │
│  Append to witness chain                 │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│  State Mutation Committed                │
│  • Action applied                        │
│  • SONA records pattern                  │
│  • DAG records execution path            │
│  • KPI monitoring activated              │
│  • Rollback prepared if KPI guard trips  │
└─────────────────────────────────────────┘
```

### 6.2 RVF Container Security

Production RLMX agents ship as security-hardened RVF containers (2.1MB reference):
- **TEE attestation**: SGX/SEV-SNP/TDX/ARM CCA verified execution
- **AIDefence**: Prompt injection, jailbreak, PII, data exfiltration detection
- **eBPF firewall**: Network policy enforcement in kernel data path
- **6-role RBAC**: Viewer, Operator, Engineer, Admin, Auditor, System
- **30-entry witness chain**: Complete, tamper-evident audit trail
- **Paranoid policy**: Default-deny, explicit capability grants only
- **COW branching**: Git-like copy-on-write for safe experimentation

---

## 7. MCP Server

```bash
# Start RLMX as MCP server (any MCP client can use it)
npx ruvector rlmx mcp start --transport streamable-http --port 8080
# Or stdio for local Claude Code integration:
npx ruvector rlmx mcp start --transport stdio
```

| MCP Tool | Description |
|---|---|
| `rlmx_query` | Query with infinite context. Auto-selects RLM or TRM strategy based on task |
| `rlmx_ingest` | Ingest data into context memory via plugin adapter |
| `rlmx_memory_stats` | Segment counts, tier distribution, storage, HNSW health |
| `rlmx_plugin_list` | List active plugins with capabilities |
| `rlmx_plugin_action` | Execute domain-specific plugin action |
| `rlmx_strategy_override` | Force RLM or TRM for next query (debugging) |
| `rlmx_trm_classify` | Run TRM classification directly (bypass scheduler) |
| `rlmx_rvf_seal` | Package current agent state as RVF container |
| `rlmx_rvf_branch` | Create COW branch for A/B experimentation |
| `rlmx_witness_chain` | Retrieve cryptographic audit trail |
| `rlmx_sona_stats` | Self-learning metrics: pattern count, improvement curve |
| `rlmx_graph_query` | Direct Cypher query against entity graph |

---

## 8. Performance Specifications

### 8.1 Development (MacBook Pro M3 Max, 128GB)

| Operation | Latency | Throughput | Notes |
|---|---|---|---|
| vllm-metal inference (Qwen3-8B Q4) | TTFT <500ms | 60-80 tok/s | MLX backend, Metal GPU |
| HNSW search (500K segments) | <0.5ms p50 | — | ruvector-core native |
| HNSW insert (batch) | — | ~8,000/sec | With embedding generation |
| TRM classification (7M params) | <5ms | — | CPU only, ONNX Runtime |
| TRM triage (50 cells) | ~250ms total | — | Sequential, no GPU |
| Full RLM recursive query (3 sub-agents) | 1-3s | — | Including vLLM calls |
| Hybrid: TRM triage + RLM deep | 1.5-3.5s | — | Only 1 RLM call for escalated cell |
| SONA adaptation | <100μs | — | Micro-LoRA delta |
| RVF cold boot | <5ms | — | Memory-mapped manifest |
| HDC hypervector bind | <50ns | — | ruvector-nervous-system |
| WTA competition (1000 neurons) | <1μs | — | ruvector-nervous-system |

### 8.2 Production (H100 + ruvector-postgres)

| Operation | Latency | Throughput | Notes |
|---|---|---|---|
| vLLM inference (Qwen3-30B FP8) | TTFT <200ms | 300-500 tok/s | H100, continuous batching |
| vLLM batch (5 parallel sub-agents) | ~200ms total | — | PagedAttention |
| HNSW search (ruvector-postgres, 1M segments) | 61μs p50 | — | PostgreSQL extension |
| Streaming ingest pipeline | — | 10,000+ seg/sec | Batch embedding + HNSW insert |
| TRM classification (7M params) | <2ms | — | H100 CUDA (overkill but available) |
| TRM triage (50 cells × 100 KPIs) | ~50ms total | — | Batched on GPU |
| Full RLM recursive query (5 sub-agents) | 200-500ms | — | End-to-end |
| Hybrid: TRM triage + RLM deep (50 cells) | 250-550ms | — | 1 RLM call for escalated cells |
| RVF federation sync | <10ms p50 | — | Cross-node state merge |
| Witness chain append | <1ms | — | Ed25519 + ML-DSA-65 |

### 8.3 WASM/Browser (Monitoring Dashboard)

| Operation | Latency | Size | Notes |
|---|---|---|---|
| rvf-wasm query | <2ms | 5.5KB | Same queries as native |
| ruvector-wasm HNSW search | <5ms | 380KB gzipped | Browser-local memory |
| rvf-solver-wasm graph analysis | <10ms | — | Client-side computation |
| TRM classification (WASM) | <20ms | ~10MB | Runs in Web Worker |
| Total dashboard bundle (no LLM) | — | ~8.5MB | Monitoring + local TRM |

---

## 9. Implementation Roadmap — Detailed Phases

### Phase 0: Foundation — RuVix + RLM Core (Weeks 1-4)

**Objective:** Prove that RLM recursive decomposition works as a RuVix scheduling policy on M3 Max with vllm-metal.

**Week 1: Environment Setup**
- Install and validate vllm-metal with Qwen3-8B-4bit on M3 Max
- Verify OpenAI-compatible API throughput (target: >50 tok/s)
- Set up ruvector-postgres Docker container
- Initialize Rust workspace with core RuVector crate dependencies
- Create `rlmx` CLI skeleton with `clap` argument parsing
- Write integration test: vLLM health check + ruvector-postgres connection

**Week 2: RuVix Kernel Bootstrap**
- Implement `vec_insert` syscall: embed text → ruvector-core HNSW insertion with metadata
- Implement `vec_search` syscall: query → ruvector-filter → ruvector-router → k-NN results
- Implement `process_fork` syscall: spawn child process with scoped capability token
- Implement `process_send`/`process_recv` syscalls: async message passing between processes
- Implement `halt_check` syscall: basic confidence threshold (nervous system integration in Phase 1)
- Write unit tests for each syscall (target: 50+ tests)

**Week 3: RLM Scheduling Policy**
- Implement RLM strategy as a RuVix scheduling policy
- Root agent receives query + memory metadata via system prompt
- Constrained action grammar: RETRIEVE, REASON, DELEGATE, COMMIT, FINAL
- Grammar-guided generation via vLLM structured output
- Sub-agent delegation via `process_fork` → vLLM batch API → `process_recv`
- Maximum recursion depth: 2 (root → sub-agent → sub-sub-agent)
- Wire up: query → RuVix scheduler → RLM strategy → vLLM → HNSW → response

**Week 4: Validation**
- Ingest a 10MB document corpus into HNSW (test: Ericsson RAN documentation)
- Run 20 long-context queries requiring multi-hop reasoning
- Compare: raw Qwen3-8B vs RLMX-on-RuVix on same queries
- Target: RLMX answers 3+ queries correctly that raw model fails
- Benchmark: end-to-end latency, HNSW search time, vLLM throughput
- Document results, identify bottlenecks for Phase 1

**Deliverables:**
- `rlmx` CLI tool: `rlmx query --prompt "..." --model http://localhost:8000`
- 6 working RuVix syscalls with 50+ tests
- RLM scheduling policy demonstrating recursive context decomposition
- Benchmark report comparing RLMX vs raw model on long-context tasks

---

### Phase 1: Cognitive Layer + TRM Integration (Weeks 5-10)

**Objective:** Integrate the full cognitive stack (attention, GNN, mincut, nervous system, SONA, DAG) and add TRM as a second scheduling policy.

**Week 5: Attention + Graph Integration**
- Integrate ruvector-attention: `attention_select` syscall selects from 39 mechanisms
- Implement heuristic for automatic mechanism selection based on query characteristics
- Integrate ruvector-gnn: `graph_query` syscall for Cypher-like entity queries
- Build entity graph from ingested context (co-occurrence → edges, entities → nodes)
- Test: query entity relationships across context segments

**Week 6: MinCut + Solver Integration**
- Integrate ruvector-mincut: `graph_cut` syscall for structural analysis
- Implement community detection on context graph (identify related segment clusters)
- Integrate ruvector-solver: `graph_diffuse` syscall for knowledge propagation
- Test: identify critical connections in a sample cell topology graph
- Test: propagate influence scores through context graph

**Week 7: Nervous System Integration**
- Integrate ruvector-nervous-system into `halt_check` syscall
- BTSP one-shot learning: agent learns new pattern from single example
- HDC hypervectors: fast concept binding for pattern matching (<50ns)
- WTA competition: resolve ambiguous classification in <1μs
- Circadian controller: schedule compute/learn/consolidate phases
- Global workspace: 4-7 item attention focus for root agent reasoning

**Week 8: SONA + DAG Self-Learning**
- Integrate SONA: micro-LoRA adaptation after each successful query
- Implement pattern bank: index successful (query, actions, result) triples in HNSW
- Integrate ruvector-dag: self-learning query optimization
- Instrument all syscalls with DAG telemetry
- Test: run 100 similar queries, measure latency reduction curve
- Target: measurable improvement (>20% latency reduction) after 100 queries

**Week 9: TRM Strategy Implementation**
- Implement TRM scheduling policy on RuVix
- TRM model loading: ONNX Runtime (CPU) or Candle (native Rust)
- Three-stream processing: question (x), answer (y), latent reasoning (z)
- Recursive refinement cycles with adaptive halting via `halt_check` syscall
- SONA integration: initialize z from pattern bank for domain-adapted starting state
- Test with generic puzzle tasks (Sudoku, maze) to validate TRM execution
- Benchmark: TRM latency per cycle, total cycles to convergence

**Week 10: Hybrid Strategy + Validation**
- Implement hybrid scheduling: TRM triage → RLM escalation
- RuVix scheduler auto-selects strategy based on task structure analysis
- Strategy selection uses ruvector-attention to analyze query characteristics
- End-to-end test: streaming data → TRM classifies → escalates → RLM analyzes
- Full cognitive stack benchmark: compare Phase 0 baseline vs Phase 1
- Document: which crate integrations provided most value

**Deliverables:**
- 12 working RuVix syscalls with 200+ tests
- TRM scheduling policy with ONNX/Candle inference
- Hybrid RLM+TRM strategy with automatic selection
- Self-learning stack (SONA + DAG) with measurable improvement metrics
- Full nervous system integration with circadian scheduling
- Cognitive stack benchmark report

---

### Phase 2: Plugin Architecture + Ericsson RAN Reference (Weeks 11-18)

**Objective:** Build the use-case-agnostic plugin system and implement the Ericsson RAN plugin as the reference implementation.

**Week 11: Plugin Trait + Loading**
- Define `DomainPlugin` trait (as specified in Section 4.1)
- Implement plugin discovery and loading:
  - Static: compiled into binary
  - Dynamic: `.so`/`.dylib` loaded at runtime via `libloading`
  - WASM: sandboxed module via `wasmtime`
  - RVF: self-contained knowledge + configuration
- Implement plugin lifecycle: init → activate → ingest → serve → deactivate
- Create plugin template generator: `rlmx plugin new my-domain`

**Week 12: Ingest Pipeline**
- Build universal ingest pipeline:
  - Plugin adapter produces `ContextSegment` stream
  - Embedding generation (all-MiniLM-L6-v2 default, plugin-overridable)
  - Delta detection (ruvector-delta-core) — skip duplicates
  - PII safety gate (cognitum-gate-kernel) — redact sensitive data
  - HNSW insertion with metadata tagging
  - Witness generation for audit trail
- Streaming mode: `rlmx ingest --plugin <name> --source <path> --watch`
- Batch mode: `rlmx ingest --plugin <name> --source <path>`

**Week 13: Safety Constraint Framework**
- Implement `SafetyConstraint` enforcement in `state_mutate` syscall
- Parameter bounds: reject mutations outside defined ranges
- KPI guards: monitor downstream metrics, auto-rollback on violation
- Rate limits: max changes per time window per entity
- Human escalation: queue action for approval when conditions met
- Test: attempt unsafe parameter change → verify rejection + reason

**Week 14: Ericsson RAN Plugin — Ingest Adapters**
- Implement `PmXmlAdapter`: parse Ericsson PM counter XML exports
- Implement `FmAlarmAdapter`: parse Fault Management alarm XML
- Implement `CmExportAdapter`: parse Configuration Management bulk exports
- Implement `CsvCounterAdapter`: generic CSV format for offline analysis
- Test: ingest real anonymized Ericsson PM data into HNSW
- Validate: segment count, embedding quality, metadata correctness

**Week 15: Ericsson RAN Plugin — Actions + Knowledge Base**
- Implement `OptimizeParameter` action extension
- Implement `FeatureLookup` action: query 588 features, 6000+ params
- Implement `ParameterValidate` action: check against 5gmo.csv spec
- Build graph schema: Cell → Site, Cell → Cell (neighbors), Cell → Feature
- Package Ericsson feature database as RVF knowledge container
- Test: feature lookup, parameter validation, graph queries on cell topology

**Week 16: Ericsson RAN Plugin — TRM Models**
- Train TRM anomaly classifier (7M params) on historical PM data:
  - Input: 50 normalized PM counters per cell per 15-min snapshot
  - Output: normal / degraded / anomalous (3-class)
  - Training: ~1000 labeled examples + augmentation
  - Target: >85% accuracy on held-out test set
- Train TRM traffic pattern recognizer (5M params):
  - Input: 96 time slots (24h × 4 per hour) of throughput values
  - Output: 8 traffic pattern archetypes (residential, office, transport hub, etc.)
  - Target: >80% accuracy
- Export both as ONNX for cross-platform inference
- Integrate into plugin via `trm_models()` trait method

**Week 17: End-to-End Ericsson RAN Validation**
- Ingest 7 days of 50-cell PM data (~50K segments)
- Run TRM triage on all cells every 15 minutes
- Verify: TRM correctly classifies known degradation events
- Trigger RLM analysis on degraded cells
- Verify: RLM correctly identifies root cause with historical context
- Verify: parameter optimization action respects safety constraints
- Verify: witness chain contains complete audit trail
- Measure: total pipeline latency, GPU utilization, memory footprint

**Week 18: Plugin Documentation + Second Plugin Skeleton**
- Write comprehensive Ericsson RAN plugin documentation
- Create "Getting Started" guide for building custom plugins
- Build skeleton second plugin (e.g., generic time-series anomaly detection)
- Validate: second plugin loads and functions with minimal code
- Code review, refactor, stabilize API

**Deliverables:**
- Complete plugin system with 4 loading mechanisms
- Ericsson RAN plugin with 6 ingest adapters, 3 action extensions, 2 TRM models
- Safety constraint framework with parameter bounds, KPI guards, rate limits
- Plugin template generator
- Second skeleton plugin proving use-case agnosticism
- End-to-end validation report on real Ericsson PM data

---

### Phase 3: RVF Containers + MCP + Production Readiness (Weeks 19-26)

**Objective:** Package RLMX agents as deployable RVF containers, expose MCP server, prepare for H100 production.

**Week 19: RVF Packaging Pipeline**
- Build `rlmx seal` command: package agent state as RVF container
  - VEC segment: HNSW index snapshot
  - INDEX segment: index metadata
  - OVERLAY segment: SONA LoRA deltas
  - GRAPH segment: entity relationship graph state
  - WITNESS segment: audit trail
  - WASM segment: 5.5KB query runtime for browser
  - MANIFEST segment: agent configuration, plugin references
- Implement `rlmx branch`: COW fork for A/B experimentation
- Implement `rlmx merge`: merge branch back after successful experiment

**Week 20: RVF Security Hardening**
- Integrate rvf-crypto: Ed25519 + ML-DSA-65 post-quantum signatures
- Implement witness chain integrity verification
- Add TEE attestation support (SGX/SEV-SNP for H100 environments)
- Implement RBAC: 6-role access control on RVF containers
- AIDefence integration: prompt injection + PII detection gates
- Test: attempt unauthorized access → verify rejection

**Week 21: MCP Server Implementation**
- Build MCP server exposing all 12 tools (Section 7)
- Streamable HTTP transport for network access
- Stdio transport for local Claude Code integration
- OAuth 2.1 authentication per MCP spec
- Rate limiting per client
- Test: Claude Code connects via stdio, executes RLMX queries

**Week 22: Federation for Multi-Node**
- Implement rvf-federation adapter for RLMX
- Scenario: two RLMX instances (different cell clusters) sync context
- Delta-based sync: only changed segments transmitted
- Conflict resolution: latest-writer-wins with witness chain ordering
- Test: two local instances sync and answer cross-cluster queries

**Week 23: WASM Monitoring Dashboard**
- Build browser dashboard using rvf-wasm + rvf-solver-wasm
- Real-time display: segment counts, query latency, SONA learning curve
- Per-plugin metrics: ingest rate, TRM classification distribution, RLM query count
- Witness chain viewer: browse audit trail
- Graph visualization: entity relationships with mincut highlighting
- Deploy as static files served by rvf-server

**Week 24: H100 Migration Testing**
- Set up H100 test environment with vLLM
- Deploy ruvector-postgres on high-memory instance
- Migrate RLMX binary: change only vLLM endpoint URL
- Verify: identical behavior, better performance
- Benchmark: compare M3 Max vs H100 across all metrics
- Load test: concurrent queries, sustained ingest, witness chain growth

**Week 25: Production Hardening**
- Implement health checks, graceful shutdown, connection pooling
- Add Prometheus metrics export for all RuVix syscalls
- Implement automatic HNSW compaction during consolidate phase
- Add backup/restore for ruvector-postgres HNSW state
- Implement log rotation for witness chains (archive to cold storage)
- Stress test: 24-hour continuous operation with simulated PM data

**Week 26: Documentation + Release Candidate**
- Write operator manual: installation, configuration, monitoring, troubleshooting
- Write plugin developer guide: trait implementation, testing, packaging
- Write architecture document: RuVix syscalls, scheduling policies, data flow
- API reference documentation (auto-generated from Rust doc comments)
- Release candidate: `rlmx v0.9.0-rc1`

**Deliverables:**
- RVF packaging pipeline with seal/branch/merge commands
- Security-hardened RVF containers with post-quantum crypto
- MCP server with 12 tools and OAuth 2.1
- Federation support for multi-node deployment
- Browser monitoring dashboard
- H100 migration validated
- Production hardening complete
- Comprehensive documentation

---

### Phase 4: Production Deployment — H100 Colocated (Weeks 27-36)

**Objective:** Deploy RLMX in production colocated with Ericsson PM data on H100 infrastructure.

**Week 27-28: Infrastructure Setup**
- Provision H100 server(s) in colocation facility
- Install vLLM with domain-tuned model (Qwen3-30B or fine-tuned variant)
- Deploy ruvector-postgres with production storage (NVMe SSD, 256GB+ RAM)
- Configure network access to Ericsson PM data source (direct or API)
- Deploy RLMX binary with Ericsson RAN plugin
- Configure TLS, firewall rules, access controls
- Verify: end-to-end data flow from PM source to RLMX to action output

**Week 29-30: Initial Data Load + Baseline**
- Ingest 30 days of historical PM data for 50-cell pilot cluster
- Verify: ~555K segments indexed, ~305MB HNSW storage
- Run TRM triage on historical data, verify classification accuracy
- Run RLM analysis on known degradation events, verify root-cause identification
- Establish baseline metrics for all KPIs in pilot cluster
- Document: current optimization state, known issues, expected improvements

**Week 31-32: Shadow Mode**
- RLMX runs alongside existing optimization system
- All RLMX recommendations logged but NOT executed
- Compare: RLMX recommendations vs actual decisions made by existing system
- Identify: cases where RLMX would have been better / worse / same
- Tune: safety constraints, confidence thresholds, TRM sensitivity

**Week 33-34: Controlled Execution**
- Enable RLMX execution for low-risk parameter changes only
- Start with: CIO adjustments on 5 cells with highest confidence
- Monitor: KPI impact via before/after comparison
- Expand: gradually increase cell count and parameter scope
- Rollback protocol tested on intentional minor misconfiguration

**Week 35-36: Full Pilot + Measurement**
- RLMX managing optimization for full 50-cell pilot cluster
- Measure against baseline (established in Weeks 29-30):
  - Handover success rate improvement
  - Throughput improvement (DL, UL, cell-edge)
  - Energy consumption reduction
  - Anomaly MTTR reduction
  - False positive rate reduction
- Generate: comprehensive pilot report with statistical significance
- Decision gate: expand to more clusters or iterate

**Deliverables:**
- Production RLMX deployment on H100 colocated infrastructure
- 30+ days of continuous operation data
- Shadow mode comparison report
- Controlled execution results
- Full pilot KPI measurement report
- Go/no-go decision for Phase 5 expansion

---

### Phase 5: Scale, Multi-Domain, and Advanced Capabilities (Weeks 37-52)

**Objective:** Scale to 1000+ cells, validate multi-domain plugin architecture, implement advanced features.

**Week 37-40: Multi-Cluster Scaling**
- Deploy RLMX federation across 3+ cell clusters (1000+ cells total)
- Each cluster has local RLMX instance with rvf-federation sync
- Coordinator instance manages cross-cluster optimization
- Implement: cross-cluster interference detection using graph_diffuse
- Validate: federation sync latency, conflict resolution, consistency

**Week 41-44: Second Domain Plugin**
- Build a complete second domain plugin (candidate: financial time-series analysis)
- Validate: same RLMX core, different plugin, different data, different actions
- Verify: no Ericsson-specific assumptions leaked into core
- Benchmark: plugin loading time, ingest throughput, query quality
- Write case study comparing both domain deployments

**Week 45-48: Advanced Capabilities**
- eBPF acceleration (rvf-ebpf): hot-path vector queries in kernel data path
  - Target: 10x throughput improvement for high-frequency TRM triage
- Kernel-level serving (rvf-kernel): self-booting RVF for air-gapped deployments
- Advanced TRM: train larger (50M param) domain-specific TRM for complex classification
- Recursive TRM: TRM that can call sub-TRM (depth 2) for multi-step structured reasoning
- Cross-domain SONA transfer: validate pattern transfer between plugins

**Week 49-52: Open Source + Community**
- Open source RLMX core under MIT/Apache-2.0 dual license
- Plugin developer documentation and tutorial series
- Plugin marketplace design (GitHub registry, Go module conventions)
- Community plugin template with CI/CD pipeline
- Conference paper: results from production RAN deployment
- Blog series: RLMX architecture deep dives

**Deliverables:**
- 1000+ cell production deployment via federation
- Second domain plugin validating use-case agnosticism
- eBPF acceleration for high-frequency workloads
- Recursive TRM capability
- Open source release
- Conference paper + documentation

---

## 10. What Makes This Architecture 15 Years Ahead

**1. Cognition kernel, not application framework.** RuVix is not a library you import. It is a kernel that thinks in vectors, graphs, proofs, and capabilities. Every AI primitive — embedding search, graph traversal, attention computation, proof verification — is a first-class kernel object accessible via syscall. This is the difference between running AI on Linux and running AI on an OS designed for AI.

**2. Dual recursive reasoning at kernel level.** RLM (large model, open-ended) and TRM (tiny model, structured) are scheduling policies, not bolted-on frameworks. The kernel dispatches to the right strategy automatically. No other system unifies these two paradigms under a single execution substrate.

**3. Self-learning at every layer, zero retraining.** SONA adapts in <100μs. DAG optimizes queries 50-80% over time. Nervous system BTSP learns from single examples. Circadian scheduling consolidates during idle periods. The system gets smarter from every interaction without any model retraining.

**4. Proof-gated mutations with quantum-ready cryptography.** Every state change requires a proof from the cognitum gate. Every proof gets a witness signed with Ed25519 (fast) and ML-DSA-65 (post-quantum). This is not an afterthought — it is the kernel's fundamental mutation primitive.

**5. One file deploys everywhere.** An RVF container boots as a Linux service (125ms), serves browser queries (5.5KB WASM), accelerates hot paths in kernel (eBPF), and federates across data centers. Model weights, HNSW index, LoRA deltas, graph state, witness chain — all in one file.

**6. Bio-inspired cognitive architecture at microsecond latencies.** Oscillatory routing (40Hz gamma), circadian compute scheduling, global workspace attention (4-7 items), winner-take-all resolution (<1μs), hyperdimensional computing (<50ns) — neuroscience primitives implemented in Rust with hardware-level performance.

**7. Domain specificity as plugins, not hardcoded logic.** The kernel knows nothing about telecom, finance, or medicine. Plugins bring data adapters, action grammars, TRM models, safety constraints, graph schemas, and knowledge bases. Adding a new domain is implementing a trait, not forking a repository.

**8. TRM + RLM hybrid eliminates 90%+ of compute waste.** Cheap TRM triage (<5ms, no GPU) screens every data point. Expensive RLM deep analysis (<500ms, GPU) runs only on escalated items. This inverts the economics of AI-driven operations monitoring.

---

## Conclusion

RLMX is not a product. It is a **cognition substrate** — a new kind of computing environment where small models reason recursively over infinite context, tiny models classify at microsecond latencies, every decision is cryptographically witnessed, and the entire system learns continuously without retraining.

The development path requires exactly one machine: a MacBook Pro M3 Max with 128GB and vllm-metal. The production path requires exactly one colocation: an H100 next to the data source. The entry point is always the same: `npx ruvector`.

The Ericsson RAN plugin is the proof of concept. But the platform is the point. Any domain where decisions must be made over unbounded temporal context — with auditability, safety constraints, and continuous improvement — is an RLMX domain.

The kernel is RuVix. The context is infinite. The models are small. The learning is continuous. The proofs are cryptographic. The deployment is one file.

**`npx ruvector rlmx init`**