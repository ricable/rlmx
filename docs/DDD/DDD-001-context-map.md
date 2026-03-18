# DDD-001: Bounded Context Map

## Overview

RLMX is a cognition kernel with 9 existing crates and 2 planned crates,
organized into 8 bounded contexts. This document maps those contexts, their
responsibilities, and integration relationships.

## Bounded Contexts

| # | Context | Type | Crate(s) | Responsibility |
|---|---------|------|----------|----------------|
| 1 | **Kernel Syscall** | Core | `rlmx-kernel` | 12-syscall dispatch, capability security, process model, proof chain |
| 2 | **Agent Lifecycle** | Core | `rlmx-agents` (planned) | 12 agent types, spawn/terminate, permission matrices |
| 3 | **Swarm Coordination** | Core | `rlmx-swarm` (planned) | 25-node cluster, zones, consensus (PBFT/Raft/Gossip), transport |
| 4 | **Inference Routing** | Core | `rlmx-kernel` (scheduler.rs), `rlmx-ruvllm`, `rlmx-rlm`, `rlmx-trm` | Strategy selection, tiered model dispatch, edge inference |
| 5 | **Research & Evolution** | Supporting | `rlmx-agents` (researcher, experimenter) | Auto-research pipeline, mutation, cross-pollination |
| 6 | **Observation & Health** | Supporting | `rlmx-cognitive` | SONA adaptation, DAG optimizer, nervous system, circadian scheduling |
| 7 | **Container & Storage** | Generic | `rlmx-rvf` | Sealed containers, COW branching, witness chains, Ed25519 signing |
| 8 | **Plugin & Domain** | Generic | `rlmx-plugin` | `DomainPlugin` trait, safety engine, ingest adapters |

## Context Map Diagram

```
 ┌─────────────────────────────────────────────────────────────────────────┐
 │                         RLMX CONTEXT MAP                               │
 │                                                                         │
 │  ┌──────────────────┐    Partnership     ┌──────────────────────┐      │
 │  │  KERNEL SYSCALL   │◄═════════════════►│   AGENT LIFECYCLE     │      │
 │  │  (rlmx-kernel)    │                   │   (rlmx-agents)       │      │
 │  │                    │                   │                       │      │
 │  │  KernelContext     │   ProcessFork     │  AgentRegistry        │      │
 │  │  Syscall (12)      │──────────────────►│  Agent (12 types)     │      │
 │  │  CapabilityToken   │   derive_child    │  AgentSpawner         │      │
 │  │  ProcessManager    │◄──────────────────│  PermissionMatrix     │      │
 │  │  ProofEngine       │                   │                       │      │
 │  └────────┬───────────┘                   └───────────┬───────────┘      │
 │           │                                           │                  │
 │   Shared  │ Kernel                          Customer/ │ Supplier         │
 │           │                                           │                  │
 │  ┌────────▼───────────┐   Upstream/Down   ┌──────────▼────────────┐     │
 │  │ INFERENCE ROUTING   │◄════════════════►│  SWARM COORDINATION    │     │
 │  │                     │                  │  (rlmx-swarm)          │     │
 │  │  Scheduler          │   route_to_node  │                        │     │
 │  │  Strategy enum      │◄────────────────│  SwarmCluster           │     │
 │  │  LocalEngine        │   load_balance   │  SwarmNode (25)        │     │
 │  │  TinyDancerRouter   │────────────────►│  Zone (A/B/C/Cld/Brw)  │     │
 │  │  (rlm, trm, ruvllm)│                  │  Consensus (PBFT/Raft) │     │
 │  └─────────────────────┘                  └────────────┬───────────┘     │
 │           │                                            │                 │
 │           │ Published Language                         │ Events          │
 │           │ (domain events)                            │                 │
 │  ┌────────▼───────────┐                   ┌───────────▼───────────┐     │
 │  │ OBSERVATION &       │   ACL            │  RESEARCH &            │     │
 │  │ HEALTH              │◄════════════════│  EVOLUTION              │     │
 │  │ (rlmx-cognitive)    │                  │  (rlmx-agents)         │     │
 │  │                     │   store_pattern  │                        │     │
 │  │  Sona / PatternBank │◄────────────────│  Researcher             │     │
 │  │  DagOptimizer       │   record_exec   │  Experimenter           │     │
 │  │  NervousSystem      │◄────────────────│  MutationStrategy       │     │
 │  │  CircadianController│                  │  CrossPollinator        │     │
 │  └─────────────────────┘                  └───────────────────────┘      │
 │           │                                                              │
 │   Conformist                                                             │
 │           │                                                              │
 │  ┌────────▼───────────┐    Open Host     ┌───────────────────────┐      │
 │  │ CONTAINER &         │    Service       │  PLUGIN & DOMAIN       │     │
 │  │ STORAGE             │◄═══════════════►│  (rlmx-plugin)         │     │
 │  │ (rlmx-rvf)          │                  │                        │     │
 │  │                     │   seal/branch    │  DomainPlugin trait     │     │
 │  │  RvfContainer       │◄────────────────│  SafetyEngine           │     │
 │  │  BranchManager      │   rvf_segments  │  IngestAdapter          │     │
 │  │  WitnessChain       │────────────────►│  DomainEvaluator        │     │
 │  │  AccessControl      │                  │                        │     │
 │  └─────────────────────┘                  └───────────────────────┘      │
 │                                                                          │
 └──────────────────────────────────────────────────────────────────────────┘
```

## Relationship Types

### Partnership (bidirectional, tight coupling)

| Pair | Nature |
|------|--------|
| Kernel Syscall <-> Agent Lifecycle | Agents are spawned via `ProcessFork` syscall with scoped `CapabilityToken`. Kernel enforces agent permissions; agents define which permissions they need. |

### Upstream / Downstream

| Upstream | Downstream | Integration |
|----------|------------|-------------|
| Swarm Coordination | Inference Routing | Swarm decides which node handles a query; Routing resolves the strategy on that node. |
| Research & Evolution | Observation & Health | Research stores successful mutations as `Pattern` entries in SONA's `PatternBank`. |

### Customer / Supplier

| Customer | Supplier | Contract |
|----------|----------|----------|
| Swarm Coordination | Agent Lifecycle | Swarm requests agents via `AgentSpawner`; Agent Lifecycle supplies them with correct capability tokens and zone placement. |

### Shared Kernel

| Contexts | Shared Types |
|----------|-------------|
| Kernel Syscall, Inference Routing | `Strategy` enum, `SchedulerConfig`, `SyscallPermission`, `ProcessId` (all in `rlmx-kernel/src/types.rs` and `scheduler.rs`) |
| Container & Storage, Plugin & Domain | `Role`, `Operation`, `AccessControl` (all in `rlmx-rvf/src/rbac.rs`) |

### Anti-Corruption Layer (ACL)

| Protected Context | External Context | ACL Purpose |
|-------------------|------------------|-------------|
| Observation & Health | Research & Evolution | `Sona::record_pattern()` accepts only `(query, actions, quality)` triples -- shields cognitive internals from research domain complexity. |
| Kernel Syscall | Swarm Coordination | Kernel never exposes `Arc<Mutex<T>>` handles directly to swarm; swarm interacts only through `Syscall` dispatch. |

### Open Host Service

| Provider | Consumers | Protocol |
|----------|-----------|----------|
| Container & Storage | Plugin & Domain, MCP Server | `RvfContainer::add_segment()`, `seal()`, `branch()` exposed as stable API. MCP tools `rlmx_rvf_seal` and `rlmx_rvf_branch` wrap these. |
| Kernel Syscall | MCP Server (`rlmx-mcp`) | JSON-RPC 2.0 over stdio or HTTP. 15 tools map to kernel syscalls through `tools.rs` handlers. |

### Conformist

| Conformist | Upstream |
|------------|----------|
| Observation & Health | Container & Storage | Cognitive layer conforms to RVF's `WitnessChain` format for audit logging. |
| MCP Server (`rlmx-mcp`) | Kernel Syscall | MCP conforms to the `Syscall` enum and `SyscallResult` variants without alteration. |

## Integration Patterns

### Domain Events (cross-context communication)

Events flow as Rust types between contexts. In the distributed swarm, these
will be serialized via `serde_json` over the swarm transport layer.

| Event | Source Context | Consumer Contexts |
|-------|---------------|-------------------|
| `SyscallDispatched` | Kernel Syscall | Observation & Health (DAG optimizer records execution) |
| `AgentSpawned` | Agent Lifecycle | Swarm Coordination (updates node agent count) |
| `QueryRouted` | Inference Routing | Observation & Health (SONA learns from routing) |
| `ExperimentCompleted` | Research & Evolution | Observation & Health (records fitness) |
| `NodeJoined` / `NodeLeft` | Swarm Coordination | Agent Lifecycle (rebalances agents) |
| `StateMutated` | Kernel Syscall | Container & Storage (appends to witness chain) |

### Shared Kernel (compiled dependency)

Crate `rlmx-kernel` is a direct Cargo dependency of `rlmx-mcp`, `rlmx-agents`,
and `rlmx-swarm`. Types like `ProcessId`, `SyscallPermission`, `Strategy`, and
`KernelError` are shared at compile time.

### Published Language (serialization boundary)

The MCP server (`rlmx-mcp`) defines the published language for external
consumers via JSON-RPC 2.0. All tool inputs and outputs are JSON. The
`McpRequest` / `McpResponse` types in `protocol.rs` form the contract.

## Crate Dependency Graph (Current + Planned)

```
rlmx-cli (binary entry point)
  +-- rlmx-kernel      (Kernel Syscall context)
  +-- rlmx-mcp         (MCP published language)
  |     +-- rlmx-kernel
  |     +-- rlmx-rvf   (Container & Storage context)
  |     +-- rlmx-ruvllm (Inference Routing - edge)
  +-- rlmx-rvf          (Container & Storage context)
  +-- rlmx-plugin        (Plugin & Domain context)
  +-- rlmx-ruvllm        (Inference Routing - edge, feature-gated)
  +-- rlmx-swarm [NEW]   (Swarm Coordination context)
  |     +-- rlmx-kernel
  |     +-- rlmx-agents
  +-- rlmx-agents [NEW]  (Agent Lifecycle + Research contexts)
  |     +-- rlmx-kernel
  |     +-- rlmx-cognitive

rlmx-rlm                 (Inference Routing - cloud, standalone)
rlmx-trm                 (Inference Routing - tiny NN, standalone)
rlmx-cognitive            (Observation & Health context, standalone)
```
