# DDD-007: Ubiquitous Language Glossary

## Overview

This glossary defines every key term used across the RLMX codebase and
documentation. Terms are grouped by bounded context. All code, comments,
commit messages, and conversations should use these terms consistently.

---

## Kernel Syscall Context

| Term | Definition |
|------|------------|
| **Syscall** | One of 12 kernel operations that an agent can invoke: `VecInsert`, `VecSearch`, `VecDelete`, `GraphQuery`, `GraphCut`, `GraphDiffuse`, `ProcessFork`, `ProcessSend`, `ProcessRecv`, `StateMutate`, `AttentionSelect`, `HaltCheck`. Defined as `enum Syscall` in `syscall.rs`. |
| **Capability Token** | An HMAC-SHA256-signed credential (`CapabilityToken`) that authorizes a process to invoke specific syscalls. Contains: owner, granted permissions, scope, expiry, and signature. |
| **Syscall Permission** | A single permission variant (`enum SyscallPermission`) granting access to one syscall family, or `All` for unrestricted access. |
| **Process Fork** | The `ProcessFork` syscall that creates a child process with a scoped capability token and isolated memory view. Analogous to Unix `fork()` but with capability narrowing. |
| **Witness Chain** | An SHA-256-chained append-only audit log (`WitnessChain`) where each entry's `content_hash` covers all its fields and its `prev_hash` links to the previous entry. |
| **Witness** | A single entry in the witness chain, recording an action hash, reasoning chain hash, evidence references, timestamp, and chain linkage. |
| **Proof** | The output of `ProofEngine::validate()`. Contains `valid: bool`, `confidence`, and the `witness_id` of the appended witness. A proof is valid when confidence meets the threshold and evidence requirements are satisfied. |
| **Proof-Gated Mutation** | The invariant that every `StateMutate` syscall must pass through `ProofEngine::validate()` and append a witness before the mutation is accepted. |
| **KernelContext** | The aggregate root holding `Arc<Mutex<T>>` handles to all subsystems (memory, graph, processes, proof engine, capability manager). All syscall dispatch flows through this struct. |
| **Process** | A kernel-level execution unit (`struct Process`) with a unique `ProcessId`, parent reference, capability token, memory scope, status, and message channel. |
| **Memory Region** | The vector memory subsystem (`MemoryRegion`) providing brute-force cosine similarity search over 64-dimensional pseudo-embeddings with Hot/Warm/Cold tier classification. |
| **Segment Tier** | Classification of a memory segment by access frequency: `Hot` (frequently accessed), `Warm` (moderate), `Cold` (rarely accessed). |
| **Graph** | The in-memory property graph (`struct Graph`) supporting Cypher-like queries, Karger/Stoer-Wagner min-cut, and heat-kernel diffusion. |
| **Dispatch** | The function `dispatch(syscall, ctx)` that pattern-matches on a `Syscall` variant and routes to the appropriate subsystem handler. The central entry point of the kernel. |

## Agent Lifecycle Context

| Term | Definition |
|------|------------|
| **Agent** | A specialized kernel process with domain-specific behavior. Distinguished from a raw `Process` by having an `AgentType`, zone assignment, model tier, and participation in the permission matrix. |
| **Agent Type** | One of 12 specializations: `Coordinator`, `Researcher`, `Router`, `Experimenter`, `Worker`, `Monitor`, `Reviewer`, `Trainer`, `Validator`, `Replicator`, `Embedder`, `Analyst`. |
| **Coordinator** | The supervisory agent type (one per zone) that orchestrates task decomposition and spawns other agents. Holds the broadest capability token in its zone. |
| **Researcher** | Agent that generates hypotheses and designs experiments for the auto-research pipeline. Max 3 concurrent. |
| **Router** | Agent that wraps the TinyDancer neural router, handling fast query classification. |
| **Experimenter** | Agent that executes experiments in COW-branched isolation. Max 8 total. |
| **Worker** | General-purpose agent that executes concrete tasks (code generation, analysis, data processing). |
| **Monitor** | Agent that performs health checks, collects metrics, and detects anomalies. |
| **Reviewer** | Agent that validates outputs against quality gates before results are accepted. |
| **Trainer** | Agent that performs online learning: SONA adaptation, LoRA updates, DAG optimizer training. |
| **Validator** | Agent that audits proof chains and validates witness integrity. |
| **Replicator** | Agent that synchronizes state across nodes in the swarm. |
| **Embedder** | Agent that generates and indexes vector embeddings. |
| **Analyst** | Agent that synthesizes results from multiple workers into coherent reports. |
| **Permission Matrix** | A 12x12 boolean table defining which agent types can send messages to which other types. Enforced on every `ProcessSend` between agents. |
| **Agent Registry** | The aggregate root (`AgentRegistry`) that tracks all live agents, enforces concurrency limits, and validates permissions. |
| **Agent Spawner** | Factory that creates agents by deriving child capability tokens and issuing `ProcessFork` syscalls. |
| **Process Group** | A logical grouping of agents assembled by a Coordinator to work on a shared task. |
| **Model Tier** | The inference capability level assigned to an agent: `Small` (Haiku-class), `Medium` (Sonnet-class), `Large` (Opus-class), `Edge` (local GGUF), `Custom`. |

## Swarm Coordination Context

| Term | Definition |
|------|------------|
| **Swarm** | The distributed cluster of up to 25 nodes running RLMX agents. |
| **Swarm Node** | A physical or virtual machine participating in the swarm. Has a `NodeId`, zone assignment, hardware profile, and health status. |
| **Zone** | A failure domain and hardware grouping. Five zones: `ZoneA` (primary), `ZoneB` (secondary), `ZoneC` (edge), `Cloud` (GPU), `Browser` (WASM). |
| **PBFT** | Practical Byzantine Fault Tolerance. Used for critical consensus (leader election). Requires `3f + 1` nodes for `f` faults. |
| **Raft** | Log replication consensus. Used for steady-state coordination within zones. Requires majority quorum (`n/2 + 1`). |
| **Gossip** | Epidemic protocol for membership propagation and health status. Eventual consistency within 10 seconds. |
| **Leader** | The elected node within a consensus group that proposes and coordinates decisions. One leader per epoch. |
| **Epoch** | A monotonically increasing counter for consensus rounds. Each leader election increments the epoch. |
| **Heartbeat** | Periodic health check message sent between nodes. 3 consecutive misses marks a node `Unreachable`. |
| **Partition** | A network split where groups of nodes cannot communicate. Detected when >50% of a zone's nodes are unreachable. |
| **Zone Failover** | Automatic migration of agents from a failed zone to its designated failover target zone. |
| **Node Profile** | Hardware description of a node: CPU cores, memory, GPU availability, backend type, OS, architecture. |

## Inference Routing Context

| Term | Definition |
|------|------------|
| **Strategy** | The inference approach for a query. Enum: `Rlm` (recursive LLM), `Trm` (tiny recursive model), `Edge` (local GGUF), `Hybrid` (triage + dispatch), `Auto` (resolved by router). |
| **Tiny-Dancer** | The planned sub-millisecond neural router based on FastGRNN. Replaces the heuristic `auto_select()` with a learned routing function. Named for its small footprint and speed. |
| **FastGRNN** | Fast, Gated Recurrent Neural Network. A compact RNN architecture suitable for real-time inference on resource-constrained devices. The core of TinyDancer. |
| **Router Input** | The 6 features extracted from a query for routing: `query_length`, `has_code`, `is_question`, `trigram_entropy`, `edge_available`, `node_load`. |
| **Escalation** | Moving a query from a lower inference tier to a higher one when confidence is below threshold. Monotonic: only upward (`Edge -> Small -> Medium -> Large -> Cloud`). |
| **Local Engine** | The `LocalEngine` struct in `rlmx-ruvllm` that wraps `CandleBackend` for GGUF model inference. Feature-gated behind `ruvllm`. |
| **Edge Inference** | Running LLM inference locally on the device (RPi5, Mac, browser) rather than calling a remote API. Uses GGUF quantized models. |
| **GGUF** | A quantized model file format used by llama.cpp and ruvllm. RLMX discovers GGUF files in `~/.rlmx/models/`. |

## Research & Evolution Context

| Term | Definition |
|------|------------|
| **Research Objective** | A goal for autonomous discovery. The aggregate root that tracks hypotheses, experiments, and the current best genome. |
| **Hypothesis** | A testable prediction generated by a Researcher agent. Contains a statement, predicted improvement, and confidence level. |
| **Experiment** | A controlled test of a hypothesis, executed by an Experimenter agent in a COW-branched environment. |
| **Genome** | A serializable configuration vector (`struct Genome`) encoding: feature weights, routing thresholds, prompt templates, and attention config. The unit of evolution. |
| **Fitness** | A multi-objective score (`FitnessScore`) combining accuracy, latency, and cost. Used to rank genomes and select survivors. |
| **Fitness Score** | `{ accuracy: f64, latency_ms: f64, cost: f64, combined: f64 }`. The `combined` field is the weighted aggregate used for comparison. |
| **Mutation** | A random modification to a genome. Types: `PointMutation` (perturb one weight), `Crossover` (combine two genomes), `Insertion` (add template), `Deletion` (remove weight). |
| **Cross-Pollination** | Transferring successful genome fragments from one research objective to another, enabling knowledge sharing across independent research tracks. |
| **Cloud Escalation** | Routing stalled research to Cloud zone nodes with GPU resources when local compute fails to make progress. Triggered by `stall_threshold`. |
| **Generation** | A monotonically increasing counter tracking evolutionary iterations within a research objective. |
| **COW Branch** | A copy-on-write fork of an RVF container (`BranchManager::create_branch()`) used to isolate experiment mutations from the main state. |

## Observation & Health Context

| Term | Definition |
|------|------------|
| **SONA** | Self-Optimizing Neural Architecture (`struct Sona`). Records successful (query, actions, result) patterns and applies micro-LoRA adaptations. The learning core of RLMX. |
| **Pattern Bank** | SONA's storage for learned patterns (`PatternBank`). Bounded capacity with lowest-quality eviction. Supports cosine similarity search over 64-dim embeddings. |
| **Pattern** | A recorded triple: `(query_embedding, actions_taken, result_quality)`. Stored in the Pattern Bank for future retrieval. |
| **Micro-LoRA** | Low-rank adaptation applied to inference pathway layers. SONA produces `LoraDelta` structs with rank-factored weight updates. Small and fast to apply. |
| **EWC++** | Elastic Weight Consolidation (improved). A regularization technique that penalizes changes to important weights, preventing catastrophic forgetting during online learning. Implemented via `FisherInformation` diagonal. |
| **DAG Optimizer** | The `DagOptimizer` that records execution history and learns optimal (strategy, attention mechanism) pairs for each query pattern. Self-improving over time. |
| **Nervous System** | The bio-inspired cognitive architecture in `rlmx-cognitive`: BTSP, HDC, WTA, Circadian Controller, Global Workspace. |
| **BTSP** | Behavioral Time-Scale Plasticity. One-shot memory system inspired by hippocampal learning. Stores patterns immediately without iterative training. |
| **Hyperdimensional Computing (HDC)** | Computing with 10,000-bit binary hypervectors. Uses XOR binding and Hamming similarity. Implemented in `HdcComputer`. |
| **Winner-Take-All (WTA)** | Competition network where neurons inhibit their neighbors. Returns the index of the winning (highest activation) neuron. |
| **Circadian Controller** | Schedules system phases by time-of-day: `Compute` (08-18h), `Learn` (18-22h), `Consolidate` (22-08h). Controls when adaptation and compaction occur. |
| **Global Workspace** | A limited-capacity attention buffer (4-7 items) inspired by Global Workspace Theory. Items compete for slots based on salience. |

## Container & Storage Context

| Term | Definition |
|------|------------|
| **RVF Container** | The sealed packaging unit (`RvfContainer`): manifest + typed segments + witness chain + optional Ed25519 signature. |
| **Seal** | Signing an RVF container with an Ed25519 key (`container.seal(signing_key)`). After sealing, the container carries a `ContainerSignature` covering the content hash. |
| **RBAC** | Role-Based Access Control. 6 roles: `Viewer`, `Operator`, `Engineer`, `Admin`, `Auditor`, `System`. Defined in `rlmx-rvf/src/rbac.rs`. |
| **Branch Manager** | Manages named COW branches of RVF containers. `create_branch()`, `merge()`, `diff()`. |
| **Segment** | A typed data unit within an RVF container (`RvfSegment`). Types: `Vec`, `Graph`, `Config`, `Model`, `Prompt`, `Evidence`. |

## Plugin & Domain Context

| Term | Definition |
|------|------------|
| **Domain Plugin** | The `DomainPlugin` trait (14 methods) in `rlmx-plugin`. The primary extension point for adding domain-specific ingest, strategy, safety, and graph schema. |
| **Safety Engine** | Enforces `ParameterBound` and `RateLimit` constraints from a plugin's `safety_constraints()`. |
| **Ingest Adapter** | A plugin-provided component that transforms domain data into kernel-compatible segments for `VecInsert`. |

---

## Anti-Patterns (terms to avoid)

| Avoid | Use Instead | Reason |
|-------|-------------|--------|
| "task" (when meaning agent work) | "syscall" or "query" | "Task" is overloaded. In RLMX, the kernel dispatches syscalls, not tasks. Agents execute queries. The `task` field on `Process` is a description string, not a first-class entity. |
| "node" (when meaning agent) | "agent" or "swarm node" | "Node" refers exclusively to a `SwarmNode` in the cluster. Agents run on nodes but are not nodes. |
| "model" (ambiguous) | "strategy", "model tier", "GGUF model", or "model spec" | Specify which model concept: the inference strategy, the capability tier, the physical GGUF file, or the `ModelSpec` configuration. |
| "token" (ambiguous) | "capability token" or "bearer token" or "auth token" | "Capability token" = `CapabilityToken` (kernel security). "Bearer token" = HTTP auth token for MCP. "Auth token" = `McpConfig.auth_token`. |
| "branch" (ambiguous) | "COW branch" or "git branch" | "COW branch" = `BranchManager` RVF fork. Use "git branch" for version control. |
| "memory" (ambiguous) | "vector memory", "memory region", "RAM", or "SONA pattern bank" | "Vector memory" = `MemoryRegion` (kernel). "SONA pattern bank" = `PatternBank` (cognitive). "RAM" = physical memory. |
| "chain" (ambiguous) | "witness chain" or "proof chain" | Always qualify. "Witness chain" = `WitnessChain` (append-only log). "Proof chain" is informal shorthand for the same thing. |
| "router" (ambiguous) | "TinyDancer router", "Router agent", or "HTTP router" | "TinyDancer" = FastGRNN neural router. "Router agent" = `AgentType::Router`. "HTTP router" = MCP server's request dispatcher. |
| "plugin" (as verb) | "extend" or "register" | Use "register a domain plugin" or "extend via `DomainPlugin` trait". |
| "run" (vague) | "dispatch", "execute", "spawn", or "generate" | Be specific: "dispatch a syscall", "execute an experiment", "spawn an agent", "generate text". |
