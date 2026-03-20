RLMX Distributed AI Agent Swarm — Implementation Plan                                                               
                                                                                                                   
 Context                             

 RLMX is a 9-crate Rust cognition kernel that currently runs as a single-node MCP server with edge inference. This
 plan transforms it into a distributed AI agent swarm with:
 - 25 physical nodes (MacBook M3 Max, 3×RPi5, 12×RPi4, 8×NUC, 1×NUC-GPU) + cloud burst
 - 12 specialized agent types with capability-secured permissions
 - Evolutionary auto-research (Karpathy-inspired, MLX on Mac, distributed across swarm)
 - FastGRNN neural model routing (tiny-dancer) replacing heuristic scheduler
 - Tiered ruvltra model inference (0.5B → 1B) with MLX/GGUF backends
 - Browser WASM compute pool for distributed matrix operations
 - Full ruvix-qemu-swarm compliance for cluster simulation and testing
 - Svelte dashboard with real-time WebSocket topology visualization

 First sprint deliverables: Auto-research on swarm + Tiny-dancer routing + Ruvltra tiered models.

 ---
 Phase 1: Swarm Foundation + Sprint 1 (Weeks 1–4)

 1.1 New Crate: crates/rlmx-swarm/

 Wraps ruvix-qemu-swarm for simulation, provides real distributed runtime.

 Cargo.toml deps: ruvix-qemu-swarm = "0.1.0" (features=["std"]), ruvix-types = "0.1.0" (features=["std"]),
 ruvector-raft = "2.0.4", ruvector-replication = "2.0.4", ruvector-cluster = "2.0.4", neuro-divergent, quinn =
 "0.11" (QUIC)

 Feature gates: default = ["process-sim"], qemu (real QEMU VMs), quic (QUIC transport)

 Files to create:

 ┌─────────────────────┬─────────────────────────────────────────────────────────────────────────────────┐
 │        File         │                                     Purpose                                     │
 ├─────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
 │ src/lib.rs          │ Re-exports, feature gates                                                       │
 ├─────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
 │ src/node.rs         │ SwarmNode: identity, zone, hardware profile, capabilities                       │
 ├─────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
 │ src/cluster.rs      │ SwarmCluster wrapping ruvector-cluster with zone-aware topology                 │
 ├─────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
 │ src/consensus.rs    │ Layered: PbftLayer (critical state), RaftLayer (metadata), GossipLayer (health) │
 ├─────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
 │ src/transport.rs    │ SwarmTransport trait: RvfTransport (QUIC) + MessageTransport (UDP gossip)       │
 ├─────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
 │ src/zone.rs         │ Zone A (Mac+NUC-GPU), Zone B (RPi5+NUC), Zone C (RPi4). Placement policy        │
 ├─────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
 │ src/health.rs       │ HealthMonitor using neuro-divergent for swarm forecasting                       │
 ├─────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
 │ src/chaos.rs        │ FaultInjector wrapping ruvix-qemu-swarm FaultType (all fault types)             │
 ├─────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
 │ src/simulation.rs   │ SimulatedSwarm: tokio tasks with in-memory channels (no QEMU)                   │
 ├─────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
 │ src/orchestrator.rs │ SwarmOrchestrator: top-level entry, configures zones, manages lifecycle         │
 ├─────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
 │ src/cloud.rs        │ SkyPilotBurst (GPU VMs), FermyonBurst (serverless), CloudPolicy                 │
 ├─────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
 │ src/browser_pool.rs │ BrowserComputePool: pull-based WebSocket work distribution                      │
 ├─────────────────────┼─────────────────────────────────────────────────────────────────────────────────┤
 │ src/types.rs        │ NodeId, ZoneId, NodeProfile, SwarmConfig, ConsensusConfig                       │
 └─────────────────────┴─────────────────────────────────────────────────────────────────────────────────┘

 1.2 Tiny-Dancer Neural Routing

 Modify: crates/rlmx-kernel/src/scheduler.rs
 Add dep: ruvector-tiny-dancer-core = "2.0.4" to rlmx-kernel

 - Add router: Option<TinyDancerRouter> field to Scheduler
 - Replace auto_select() body: if router present, run FastGRNN forward pass (<1ms), else fall back to heuristic
 - Add Strategy::Swarm variant for distributed queries
 - Delete old heuristic tests, write FastGRNN router tests

 New file: crates/rlmx-kernel/src/router.rs
 - TinyDancerRouter wrapping FastGRNN
 - RouterInput (query length, has_code, is_question, trigram entropy, edge_available, node load)
 - RouterOutput (softmax over [Rlm, Trm, Edge, Hybrid, Swarm])
 - train_from_history() for online learning from (features, strategy, reward) triples

 1.3 Ruvltra Tiered Model Integration

 Modify: crates/rlmx-ruvllm/

 config.rs changes:
 - Add ModelTier enum: Small, ClaudeCode, Medium, Custom(String)
 - Add TieredConfig with models: Vec<(ModelTier, ModelSpec)>, escalation_threshold: f32
 - Add InferenceBackendType enum: Candle, Mlx, Remote(String)

 New file: crates/rlmx-ruvllm/src/tiered.rs
 - TieredEngine: holds multiple ModelSpec, routes via tiny-dancer
 - escalate(): if confidence below threshold, re-run on next tier
 - MlxSubprocess: shells out to Python MLX via tokio::process::Command, JSON stdin/stdout

 engine.rs changes:
 - Add TieredEngine alongside LocalEngine
 - LocalEngine::generate() returns confidence score alongside text
 - EngineKind enum wrapping both

 New file: deploy/mlx_bridge.py — Python script wrapping autoresearch-mlx training loop as MCP tool

 1.4 New Crate: crates/rlmx-agents/

 Cargo.toml deps: rlmx-kernel, rlmx-swarm, rlmx-ruvllm, rlmx-cognitive, ruvector-sona = "0.1.6", neuro-divergent,
 wasmtime = "27" (optional, feature = "wasm")

 Files to create:

 ┌─────────────────────┬────────────────────────────────────────────────────────────────┐
 │        File         │                            Purpose                             │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/lib.rs          │ Re-exports                                                     │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/types.rs        │ AgentType enum (12 variants), AgentConfig, AgentMessage        │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/registry.rs     │ 12×12 permission matrix, maps agent type → syscall permissions │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/spawn.rs        │ AgentSpawner: ProcessFork with scoped capability tokens        │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/researcher.rs   │ Auto-research: hypothesize, spawn experimenters, synthesize    │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/router_agent.rs │ Route queries via tiny-dancer to best node/agent               │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/coordinator.rs  │ Swarm lifecycle, PBFT/Raft consensus leader (PID 0)            │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/worker.rs       │ Execute inference tasks from Router                            │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/monitor.rs      │ Health watching via neuro-divergent, anomaly alerts            │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/reviewer.rs     │ Validate outputs, experiment designs, state mutations          │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/trainer.rs      │ MLX/CUDA training, SONA micro-LoRA, checkpoint mgmt            │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/validator.rs    │ Walk witness chain, verify proof integrity                     │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/replicator.rs   │ Cross-zone data sync via RVF containers                        │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/embedder.rs     │ Real neural embeddings replacing hash-based pseudo-embeddings  │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/analyst.rs      │ Graph min-cut, diffusion, community detection                  │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/experimenter.rs │ Parameter search, COW-branched experiments                     │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/mutation.rs     │ Evolutionary engine: mutate, cross-pollinate, cloud escalation │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/experiment.rs   │ Experiment struct: hypothesis, evidence, fitness, generation   │
 ├─────────────────────┼────────────────────────────────────────────────────────────────┤
 │ src/wasm_bridge.rs  │ WASM skill loading via wasmtime (feature-gated)                │
 └─────────────────────┴────────────────────────────────────────────────────────────────┘

 1.5 Workspace Cargo.toml Changes

 Add workspace members: crates/rlmx-swarm, crates/rlmx-agents

 Add [workspace.dependencies]:
 ruvix-qemu-swarm = { version = "0.1.0", features = ["std"] }
 ruvix-types = { version = "0.1.0", features = ["std"] }
 ruvector-tiny-dancer-core = { version = "2.0.4", features = ["std"] }
 ruvector-raft = "2.0.4"
 ruvector-replication = "2.0.4"
 ruvector-cluster = "2.0.4"
 ruvector-sona = { version = "0.1.6", features = ["std"] }
 ruvector-dag = "2.0.4"
 ruvector-nervous-system = { version = "2.0.4", features = ["std"] }
 ruvector-attention = { version = "2.0.4", features = ["std"] }
 ruvector-mincut = "2.0.4"
 neuro-divergent = "0.1"
 quinn = "0.11"
 tokio-tungstenite = "0.24"

 1.6 CLI Additions (crates/rlmx-cli/src/main.rs)

 New subcommands:
 - rlmx swarm start --zone A --port 9000 — start local swarm node
 - rlmx swarm status / topology / chaos — swarm management
 - rlmx agent spawn --type researcher --task "..." / list / kill — agent lifecycle
 - rlmx research start --topic "..." --nodes 3 / status / list — auto-research

 1.7 MCP Tool Expansion (crates/rlmx-mcp/src/tools.rs)

 10 new tools alongside existing 15:

 ┌───────────────────────┬───────────┬─────────────────────────────────────────┐
 │         Tool          │   RBAC    │                 Purpose                 │
 ├───────────────────────┼───────────┼─────────────────────────────────────────┤
 │ rlmx_swarm_status     │ Viewer+   │ Cluster topology, per-node health       │
 ├───────────────────────┼───────────┼─────────────────────────────────────────┤
 │ rlmx_swarm_topology   │ Viewer+   │ Zone map, node assignments              │
 ├───────────────────────┼───────────┼─────────────────────────────────────────┤
 │ rlmx_agent_spawn      │ Engineer+ │ Create agent via ProcessFork            │
 ├───────────────────────┼───────────┼─────────────────────────────────────────┤
 │ rlmx_agent_list       │ Operator+ │ List running agents                     │
 ├───────────────────────┼───────────┼─────────────────────────────────────────┤
 │ rlmx_agent_terminate  │ Admin+    │ Kill agent process                      │
 ├───────────────────────┼───────────┼─────────────────────────────────────────┤
 │ rlmx_research_start   │ Engineer+ │ Start auto-research experiment          │
 ├───────────────────────┼───────────┼─────────────────────────────────────────┤
 │ rlmx_research_status  │ Operator+ │ Experiment progress                     │
 ├───────────────────────┼───────────┼─────────────────────────────────────────┤
 │ rlmx_experiment_list  │ Viewer+   │ All active experiments                  │
 ├───────────────────────┼───────────┼─────────────────────────────────────────┤
 │ rlmx_mutation_history │ Viewer+   │ Evolutionary mutation log               │
 ├───────────────────────┼───────────┼─────────────────────────────────────────┤
 │ rlmx_forecast         │ Operator+ │ neuro-divergent forecasting             │
 ├───────────────────────┼───────────┼─────────────────────────────────────────┤
 │ rlmx_train            │ Engineer+ │ MCP tool wrapping MLX Python subprocess │
 └───────────────────────┴───────────┴─────────────────────────────────────────┘

 1.8 WebSocket Server (crates/rlmx-mcp/src/ws.rs)

 New file using tokio-tungstenite. Broadcasts: NodeJoined, NodeLeft, AgentSpawned, AgentTerminated, HealthUpdate,
 ExperimentUpdate, MutationFound. Clients subscribe to event types via JSON messages. Port 3001 alongside MCP on
 3000.

 ---
 Phase 2: Kernel Replacement with Ruvix (Weeks 5–8)

 2.1 Capability System → ruvix-cap

 File: crates/rlmx-kernel/src/capability.rs
 - Replace HMAC-SHA256 CapabilityToken with newtype wrapping ruvix_cap::Capability
 - Replace CapabilityManager with wrapper around ruvix_cap::CapabilitySpace
 - Implement From<SyscallPermission> for ruvix-cap permission type
 - Remove hmac/sha2 deps from kernel

 2.2 Cognitive Layer → Ruvector Crates

 Files: crates/rlmx-cognitive/src/{sona.rs, dag.rs, nervous.rs, attention.rs}
 - Replace all with thin wrappers around ruvector-sona, ruvector-dag, ruvector-nervous-system, ruvector-attention
 - Remove ndarray dep, add ruvector deps

 2.3 Graph Min-Cut → ruvector-mincut

 File: crates/rlmx-kernel/src/graph.rs
 - Replace Karger/Stoer-Wagner with ruvector_mincut calls
 - Keep Graph struct and cypher_query() parser

 2.4 Kernel Types Bridge

 File: crates/rlmx-kernel/src/types.rs
 - Add ruvix-types dep, implement From/Into conversions for shared types

 ---
 Phase 3: Full Agent System (Weeks 9–12)

 3.1 Agent Types & Permission Matrix

 12 agents × 12 syscalls. Key constraints:

 ┌──────────────┬────────────────────┬───────────────────┬─────────────┬─────────┐
 │    Agent     │     Can Fork?      │ Can Mutate State? │    Model    │  Zone   │
 ├──────────────┼────────────────────┼───────────────────┼─────────────┼─────────┤
 │ Coordinator  │ Yes (any)          │ Yes               │ medium      │ A       │
 ├──────────────┼────────────────────┼───────────────────┼─────────────┼─────────┤
 │ Researcher   │ Yes (Experimenter) │ Yes               │ claude-code │ A       │
 ├──────────────┼────────────────────┼───────────────────┼─────────────┼─────────┤
 │ Router       │ Yes (Worker)       │ No                │ small       │ B       │
 ├──────────────┼────────────────────┼───────────────────┼─────────────┼─────────┤
 │ Experimenter │ Yes (sub-Exp)      │ Yes               │ medium      │ A/Cloud │
 ├──────────────┼────────────────────┼───────────────────┼─────────────┼─────────┤
 │ Worker       │ No                 │ Yes               │ tiered      │ Multi   │
 ├──────────────┼────────────────────┼───────────────────┼─────────────┼─────────┤
 │ Monitor      │ No                 │ Yes               │ small       │ C       │
 ├──────────────┼────────────────────┼───────────────────┼─────────────┼─────────┤
 │ Reviewer     │ No                 │ Yes               │ claude-code │ A       │
 ├──────────────┼────────────────────┼───────────────────┼─────────────┼─────────┤
 │ Trainer      │ No                 │ Yes               │ medium      │ A(GPU)  │
 ├──────────────┼────────────────────┼───────────────────┼─────────────┼─────────┤
 │ Validator    │ No                 │ No                │ small       │ C       │
 ├──────────────┼────────────────────┼───────────────────┼─────────────┼─────────┤
 │ Replicator   │ No                 │ Yes               │ small       │ B       │
 ├──────────────┼────────────────────┼───────────────────┼─────────────┼─────────┤
 │ Embedder     │ No                 │ No                │ small       │ A/B     │
 ├──────────────┼────────────────────┼───────────────────┼─────────────┼─────────┤
 │ Analyst      │ No                 │ Yes               │ medium      │ A       │
 └──────────────┴────────────────────┴───────────────────┴─────────────┴─────────┘

 3.2 Agent Hierarchy (Spawn Tree)

 Coordinator (PID 0, All permissions, Raft leader)
 ├── Router (1/zone, spawns Workers)
 │   └── Worker (pool 1-N, auto-terminate 5min idle)
 ├── Monitor (1/zone, watchdog, never terminates)
 ├── Replicator (1/zone-pair, continuous sync)
 ├── Embedder (1/compute-zone, always running)
 ├── Validator (periodic or on-demand)
 ├── Reviewer (on-demand, max 2)
 ├── Analyst (on-demand, max 2)
 ├── Researcher (on objective, max 3)
 │   └── Experimenter (per hypothesis, max 8 total)
 │       └── sub-Experimenter (grid search depth ≤2)
 └── Trainer (on job, max 1/GPU-zone)

 3.3 Process Model Extension

 File: crates/rlmx-kernel/src/process.rs
 - Add agent_type: Option<AgentType> to Process
 - Add spawn_agent() to ProcessManager (validates agent type × parent permissions × max concurrent)
 - Add ProcessGroup for agent teams

 3.4 SKILL.md Definitions

 Create crates/rlmx-agents/skills/ with 12 SKILL.md files defining each agent's system prompt, tool access,
 constraints, and behavioral rules. Format:
 ---
 name: rlmx-{type}
 model: {ruvltra-small|claude-code|medium}
 tools: [tool_list]
 zone: {A|B|C|Multi|Cloud}
 ---

 ---
 Phase 4: Auto-Research + Evolutionary Swarm (Weeks 13–16)

 4.1 Evolutionary Mutation Engine

 File: crates/rlmx-agents/src/mutation.rs
 - MutationStrategy: serializable genome (feature weights, routing thresholds, prompt templates)
 - mutate(): random perturbations to strategy genome
 - CrossPollinator: receives mutations via Gossip, adopts if fitness improves (peer sharing)
 - CloudEscalation: when swarm stalls (no improvement N rounds), escalate to SkyPilot GPU VM
 - FitnessEvaluator: scores (accuracy, latency, cost) tradeoff
 - Integration with SONA PatternBank: successful mutations become patterns

 4.2 Auto-Research Flow

 1. Coordinator receives research objective, spawns Researcher
 2. Researcher searches prior patterns (VecSearch + SONA), generates N hypotheses
 3. Researcher spawns N Experimenters (each gets COW-branched state via rlmx_rvf_branch)
 4. Experimenters each independently mutate train.py using local ruvltra model
 5. Experimenters request Trainer spawn via Coordinator (MLX on Mac, CUDA on NUC-GPU)
 6. Trainer runs 5-min training experiment, reports val_bpb
 7. Cross-pollination via Gossip: successful mutations propagate
 8. Cloud escalation if entire swarm stalls
 9. Researcher synthesizes findings, Reviewer validates
 10. Coordinator integrates winning configuration, Replicator syncs

 4.3 Neuro-Divergent Forecasting

 Three targets: swarm health prediction, query load forecasting, mutation quality prediction. Exposed as
 rlmx_forecast MCP tool.

 ---
 Phase 5: Dashboard + Browser Pool (Weeks 17–20)

 5.1 Svelte Dashboard (frontend/)

 Replace vanilla JS with Svelte + Vite. Structure:

 frontend/
   package.json, svelte.config.js, vite.config.js
   src/
     App.svelte
     lib/ws.ts, mcp.ts, types.ts
     components/
       TopologyMap.svelte         # D3.js zone topology
       NodeDetail.svelte          # Per-node drill-down
       AgentList.svelte           # Running agents table
       ExperimentTracker.svelte   # Auto-research timeline
       MutationGraph.svelte       # Evolutionary fitness graph
       ComputePool.svelte         # Browser WASM worker status
       QueryConsole.svelte        # Interactive MCP queries
       HealthForecast.svelte      # neuro-divergent charts
       WitnessChain.svelte        # Proof audit trail
       ModelStatus.svelte         # Tiered ruvltra status
   static/ruvllm-wasm/            # Existing WASM bundle

 Dual channel: WebSocket (ws://host:3001/ws) for real-time push, HTTP MCP (POST /mcp) for tool calls.

 5.2 Browser WASM Compute Pool

 Pull-based: browser workers poll via WebSocket for work units (embedding, matmul). Server queues tasks, workers
 compute and return results. Tab close → work re-queued. Uses existing frontend/ruvllm-wasm/ bundle + Web Workers +
 SharedArrayBuffer.

 ---
 Hardware Topology

 Hierarchical Zones

 ┌─────────┬─────────────────────────────────────────┬─────────────────────────────────────────────┬────────────┐
 │  Zone   │                  Nodes                  │                    Role                     │ Consensus  │
 ├─────────┼─────────────────────────────────────────┼─────────────────────────────────────────────┼────────────┤
 │ A       │ Mac M3 Max 128GB + NUC-GPU (GTX1060     │ Compute: training, complex inference,       │ PBFT       │
 │         │ 6GB)                                    │ coordination                                │ leader     │
 ├─────────┼─────────────────────────────────────────┼─────────────────────────────────────────────┼────────────┤
 │ B       │ 3×RPi5 + 8×NUC                          │ Inference: edge serving, routing, data      │ Raft       │
 │         │                                         │ relay                                       │ voters     │
 ├─────────┼─────────────────────────────────────────┼─────────────────────────────────────────────┼────────────┤
 │ C       │ 12×RPi4 (heterogeneous RAM)             │ Edge: monitoring, validation, vector        │ Gossip     │
 │         │                                         │ shards, relay                               │            │
 ├─────────┼─────────────────────────────────────────┼─────────────────────────────────────────────┼────────────┤
 │ Cloud   │ SkyPilot VMs, Fermyon Spin, Supabase,   │ Burst: GPU training, serverless inference,  │ On-demand  │
 │         │ HF jobs                                 │ batch                                       │            │
 ├─────────┼─────────────────────────────────────────┼─────────────────────────────────────────────┼────────────┤
 │ Browser │ WASM compute pool                       │ Distributed: matmul, embeddings,            │ Stateless  │
 │         │                                         │ pre-routing                                 │            │
 └─────────┴─────────────────────────────────────────┴─────────────────────────────────────────────┴────────────┘

 RPi4 Per-Node Config (auto-detected at boot)

 ┌─────┬──────────────────────────────────────┬────────────────────────────────┐
 │ RAM │                 Role                 │             Agents             │
 ├─────┼──────────────────────────────────────┼────────────────────────────────┤
 │ 4GB │ Small model inference + vector shard │ Worker (edge), Monitor         │
 ├─────┼──────────────────────────────────────┼────────────────────────────────┤
 │ 2GB │ Vector memory shard + relay          │ Replicator endpoint, Validator │
 ├─────┼──────────────────────────────────────┼────────────────────────────────┤
 │ 1GB │ Gossip relay + metrics collector     │ Monitor (health relay only)    │
 └─────┴──────────────────────────────────────┴────────────────────────────────┘

 NUC-GPU Triple Role

 1. Training (primary): auto-research experiments via CUDA/PyTorch
 2. Inference (secondary): ruvltra-medium (1B) at higher throughput
 3. Embedding (tertiary): real 768-dim embeddings for entire swarm

 Cloud Burst Triggers

 ┌──────────────────────────────────────────┬────────────────────────────────────────┬──────────────────────┐
 │                 Trigger                  │              Cloud Target              │       Protocol       │
 ├──────────────────────────────────────────┼────────────────────────────────────────┼──────────────────────┤
 │ Auto-research parallelism                │ SkyPilot GPU VMs                       │ Training jobs        │
 ├──────────────────────────────────────────┼────────────────────────────────────────┼──────────────────────┤
 │ Inference overflow (latency > threshold) │ Fermyon Spin / Supabase edge functions │ Serverless inference │
 ├──────────────────────────────────────────┼────────────────────────────────────────┼──────────────────────┤
 │ Batch tasks (model conversion, eval)     │ HuggingFace jobs via hf CLI            │ Scheduled            │
 └──────────────────────────────────────────┴────────────────────────────────────────┴──────────────────────┘

 ---
 QEMU Swarm Test Scenarios

 Integration test crate: crates/rlmx-swarm/tests/

 ┌────────────────────────────────────────┬──────────────────┬─────────────────────────────────────┐
 │                  Test                  │      Fault       │             Validation              │
 ├────────────────────────────────────────┼──────────────────┼─────────────────────────────────────┤
 │ test_3_node_pbft_consensus             │ None             │ 3 nodes agree on state mutation     │
 ├────────────────────────────────────────┼──────────────────┼─────────────────────────────────────┤
 │ test_5_node_raft_leader_election       │ Leader crash     │ New leader elected <2s              │
 ├────────────────────────────────────────┼──────────────────┼─────────────────────────────────────┤
 │ test_network_partition_gossip_recovery │ NetworkPartition │ Gossip reconverges <10s after heal  │
 ├────────────────────────────────────────┼──────────────────┼─────────────────────────────────────┤
 │ test_byzantine_node_rejected           │ Byzantine        │ PBFT rejects contradictory votes    │
 ├────────────────────────────────────────┼──────────────────┼─────────────────────────────────────┤
 │ test_zone_failover                     │ Zone A crash     │ Zone B takes over compute           │
 ├────────────────────────────────────────┼──────────────────┼─────────────────────────────────────┤
 │ test_evolutionary_mutation_propagation │ None             │ Mutation gossips across 5 nodes <5s │
 ├────────────────────────────────────────┼──────────────────┼─────────────────────────────────────┤
 │ test_tiered_model_escalation           │ None             │ Small fails → medium succeeds       │
 ├────────────────────────────────────────┼──────────────────┼─────────────────────────────────────┤
 │ test_chaos_all_faults                  │ Random all       │ Swarm recovers to healthy <30s      │
 └────────────────────────────────────────┴──────────────────┴─────────────────────────────────────┘

 Default: SimulatedSwarm (in-memory channels). --features qemu: real QEMU VMs.

 ---
 rlmx-plan.md Structure

 Create /Users/cedric/rlmx/rlmx-plan.md with narrative + specs:

 1. Vision — Distributed AI agent swarm with self-learning
 2. Architecture — Hierarchical zones, layered consensus, RVF+QUIC transport
 3. Agent System — 12 types, permission matrix, spawn hierarchy, SKILL.md definitions
 4. Auto-Research — Evolutionary swarm, cross-pollination, cloud escalation
 5. Model Routing — Tiny-dancer FastGRNN, tiered ruvltra, MLX+GGUF backends
 6. Hardware — 25-node topology, per-node config, cloud burst
 7. Frontend — Svelte dashboard, WebSocket events, WASM compute pool
 8. Ruvnet Crate Integration Map — Which of 120+ crates map to which subsystem
 9. Sprint Plan — Phase milestones with first sprint: auto-research + routing + ruvltra

 ---
 Sprint 1 Milestone Checklist

 1. cargo build --workspace succeeds with rlmx-swarm and rlmx-agents added
 2. SimulatedSwarm::new(3) creates 3 process-simulated nodes with Gossip health
 3. TinyDancerRouter::route(query) returns Strategy via FastGRNN forward pass
 4. TieredEngine::generate(prompt) runs ruvltra-small, escalates if confidence low
 5. ResearcherAgent::research(topic) spawns via ProcessFork, queries memory, stores findings
 6. rlmx swarm start --zone A --port 9000 starts a local swarm node
 7. rlmx agent spawn --type researcher --task "investigate X" spawns agent
 8. rlmx_swarm_status MCP tool returns cluster state
 9. 3-node PBFT consensus test passes in process-sim mode
 10. Existing tests replaced where modules changed, new test count ≥ old

 ---
 Verification

 # Build everything (stub build must pass)
 cargo build --workspace
 cargo build --workspace --features ruvllm,metal

 # Run all tests
 cargo test --workspace

 # Start simulated 3-node swarm
 cargo run -p rlmx-cli -- swarm start --zone A --port 9000 &
 cargo run -p rlmx-cli -- swarm start --zone B --port 9001 &
 cargo run -p rlmx-cli -- swarm start --zone C --port 9002 &
 cargo run -p rlmx-cli -- swarm status

 # Test neural routing
 cargo run -p rlmx-cli -- query "hello" --strategy auto  # → Edge via tiny-dancer
 cargo run -p rlmx-cli -- query "implement a B-tree" --strategy auto  # → Rlm/Trm

 # Test tiered inference
 RLMX_EDGE_MODEL=ruvltra-small cargo run -p rlmx-cli -- edge generate --prompt "hello"

 # Spawn research agent
 cargo run -p rlmx-cli -- agent spawn --type researcher --task "optimize edge latency"

 # Run swarm tests
 cargo test -p rlmx-swarm

 # Lint + format
 cargo clippy --workspace && cargo fmt --check

 ---
 Critical Files Summary

 ┌─────────────────────────────────────────────────────────┬─────────┬───────────────────────────────────────┐
 │                          File                           │ Action  │             What Changes              │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/Cargo.toml                           │ Modify  │ Add workspace members + deps          │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/crates/rlmx-kernel/src/scheduler.rs  │ Modify  │ Replace heuristics with tiny-dancer   │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/crates/rlmx-kernel/src/router.rs     │ Create  │ TinyDancerRouter + FastGRNN wrapper   │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/crates/rlmx-kernel/src/capability.rs │ Modify  │ Replace with ruvix-cap (Phase 2)      │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/crates/rlmx-kernel/src/process.rs    │ Modify  │ Add agent_type, spawn_agent()         │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/crates/rlmx-kernel/src/types.rs      │ Modify  │ Add AgentType, HardwareZone enums     │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/crates/rlmx-kernel/src/graph.rs      │ Modify  │ Replace min-cut with ruvector-mincut  │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/crates/rlmx-ruvllm/src/tiered.rs     │ Create  │ TieredEngine + MlxSubprocess          │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/crates/rlmx-ruvllm/src/engine.rs     │ Modify  │ Add EngineKind, confidence scores     │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/crates/rlmx-ruvllm/src/config.rs     │ Modify  │ Add ModelTier, TieredConfig           │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/crates/rlmx-mcp/src/tools.rs         │ Modify  │ Add 11 new swarm/agent/research tools │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/crates/rlmx-mcp/src/ws.rs            │ Create  │ WebSocket server for real-time push   │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/crates/rlmx-cognitive/src/*.rs       │ Modify  │ Replace with ruvector crate wrappers  │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/crates/rlmx-cli/src/main.rs          │ Modify  │ Add swarm/agent/research subcommands  │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/crates/rlmx-swarm/                   │ Create  │ Entire new crate (13 files)           │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/crates/rlmx-agents/                  │ Create  │ Entire new crate (19 files)           │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/deploy/mlx_bridge.py                 │ Create  │ Python MLX training MCP wrapper       │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/frontend/                            │ Rewrite │ Svelte dashboard (Phase 5)            │
 ├─────────────────────────────────────────────────────────┼─────────┼───────────────────────────────────────┤
 │ /Users/cedric/rlmx/rlmx-plan.md                         │ Create  │ Narrative + specs document            │
 └─────────────────────────────────────────────────────────┴─────────┴───────────────────────────────────────┘