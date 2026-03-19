# DDD-001: Bounded Context Map

## Overview

RLMX is a cognition kernel with 19 crates organized into 14 bounded contexts.
This document maps those contexts, their responsibilities, and integration
relationships.

## Bounded Contexts

| # | Context | Type | Crate(s) | Responsibility |
|---|---------|------|----------|----------------|
| 1 | **Kernel Syscall** | Core | `rlmx-kernel` | 17-syscall-permission dispatch, capability security, process model, proof chain |
| 2 | **Agent Lifecycle** | Core | `rlmx-agents` | 17 agent types, spawn/terminate, 17x17 permission matrix |
| 3 | **Swarm Coordination** | Core | `rlmx-swarm` | 6-zone cluster, consensus (PBFT/Raft/Gossip), transport, discovery |
| 4 | **Inference Routing** | Core | `rlmx-kernel` (scheduler.rs), `rlmx-ruvllm`, `rlmx-rlm`, `rlmx-trm` | Strategy selection, tiered model dispatch, edge inference |
| 5 | **Research & Evolution** | Supporting | `rlmx-agents` (researcher, experimenter) | Auto-research pipeline, mutation, cross-pollination |
| 6 | **Observation & Health** | Supporting | `rlmx-cognitive` | SONA adaptation, DAG optimizer, nervous system, circadian scheduling |
| 7 | **Container & Storage** | Generic | `rlmx-rvf` | Sealed containers, COW branching, witness chains, Ed25519 signing |
| 8 | **Plugin & Domain** | Generic | `rlmx-plugin` | `DomainPlugin` trait, safety engine, ingest adapters |
| 9 | **Voice Interaction** | Core | `rlmx-voice` (DDD-008) | Voice pipeline: VAD, STT, intent decomposition, TTS, session memory |
| 10 | **Phone Runtime** | Core | `rlmx-phone` (DDD-009) | Mobile command center: background scheduling, notifications, widgets, engagement (Life Score, streaks, gamification) |
| 11 | **Agent Marketplace** | Supporting | `rlmx-marketplace` (DDD-010) | Agent ecosystem: publishing, discovery, installation, billing, security review, developer SDK |
| 12 | **Personal Mesh** | Core | `rlmx-mesh` (DDD-011) | Multi-device fleet: device registry, state sync, discovery (mDNS/BLE), failover & degradation |
| 13 | **Federated Learning** | Supporting | `rlmx-federation` (DDD-012) | Privacy-preserving learning: weekly cycles, on-device anonymization, cloud aggregation, LoRA distribution |
| 14 | **Subscription Billing** | Supporting | `rlmx-billing` (DDD-013) | 6-tier subscriptions, family plans, developer rev share, usage tracking, capability enforcement |

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
 │  ═══════════════════════ Voice-First Contexts ═══════════════════════    │
 │                                                                          │
 │  ┌──────────────────────┐  Partnership   ┌──────────────────────┐       │
 │  │ VOICE INTERACTION     │◄═════════════►│  PHONE RUNTIME        │       │
 │  │ (rlmx-voice)          │               │  (rlmx-phone)         │       │
 │  │ DDD-008               │               │  DDD-009              │       │
 │  │                       │               │                       │       │
 │  │  VoicePipeline        │  voice runs   │  BackgroundScheduler  │       │
 │  │  VadDetector          │  within phone  │  NotificationManager  │       │
 │  │  SttTranscriber       │               │  WidgetEngine         │       │
 │  │  IntentDecomposer     │               │  LifeScoreTracker     │       │
 │  │  TtsSynthesizer       │               │  StreakEngine         │       │
 │  │  SessionMemory        │               │  GamificationModule   │       │
 │  └──────┬──────┬─────────┘               └──────┬──────┬─────────┘       │
 │         │      │                                │      │                 │
 │ Upstream│      │ Upstream              Upstream │      │ Downstream      │
 │ (dispatches    │ (18-dim voice     (starts/stops│      │ (installs       │
 │  VoiceTranscr  │  features)         agents)     │      │  agents)        │
 │  VoiceSynth    │                                │      │                 │
 │  IntentRoute)  │                                │      │                 │
 │         │      │                                │      │                 │
 │         ▼      ▼                                ▼      │                 │
 │  ┌──────────────────┐  ┌────────────────┐  ┌───────────┴───────────┐    │
 │  │  KERNEL SYSCALL   │  │ INFERENCE      │  │  AGENT LIFECYCLE      │    │
 │  │  (rlmx-kernel)    │  │ ROUTING        │  │  (rlmx-agents)        │    │
 │  │                    │  │ (scheduler.rs) │  │                       │    │
 │  └──────────────────┘  └────────────────┘  └───────────────────────┘    │
 │                                                        ▲                 │
 │         ┌──────────────────────┐  Partnership          │                 │
 │         │ AGENT MARKETPLACE     │◄════════════════════►│                 │
 │         │ (rlmx-marketplace)    │  publishes agents                      │
 │         │ DDD-010               │  that lifecycle manages                │
 │         │                       │                                        │
 │         │  AgentPublisher       │  Upstream to                           │
 │         │  DiscoveryIndex       │  Phone Runtime                         │
 │         │  InstallManager       │─────────────────────►PHONE RUNTIME     │
 │         │  BillingEngine        │  (phone installs                       │
 │         │  SecurityReviewer     │   marketplace agents)                  │
 │         │  DeveloperSdk         │                                        │
 │         └──────────────────────┘                                         │
 │                                                                          │
 │  Additional Swarm relationship:                                          │
 │    PHONE RUNTIME ──(upstream)──► SWARM COORDINATION                      │
 │    Phone joins as Zone A-Mobile node                                     │
 │                                                                          │
 └──────────────────────────────────────────────────────────────────────────┘
```

## Relationship Types

### Partnership (bidirectional, tight coupling)

| Pair | Nature |
|------|--------|
| Kernel Syscall <-> Agent Lifecycle | Agents are spawned via `ProcessFork` syscall with scoped `CapabilityToken`. Kernel enforces agent permissions; agents define which permissions they need. |
| Voice Interaction <-> Phone Runtime | Voice pipeline runs within the phone runtime. Phone provides audio capture and playback; Voice provides transcription and synthesis. Shared session context. |
| Agent Marketplace <-> Agent Lifecycle | Marketplace publishes agent packages that Agent Lifecycle instantiates. Lifecycle reports agent health back to Marketplace for quality scoring. |
| Personal Mesh <-> Swarm Coordination | Mesh manages device fleet; Swarm manages agent coordination across those devices. Mesh provides device capabilities; Swarm assigns work units. |
| Subscription Billing <-> Agent Marketplace | Billing enforces tier limits on marketplace operations (publishing requires Developer tier). Marketplace reports revenue for developer payouts. |

### Upstream / Downstream

| Upstream | Downstream | Integration |
|----------|------------|-------------|
| Swarm Coordination | Inference Routing | Swarm decides which node handles a query; Routing resolves the strategy on that node. |
| Research & Evolution | Observation & Health | Research stores successful mutations as `Pattern` entries in SONA's `PatternBank`. |
| Voice Interaction | Kernel Syscall | Voice dispatches `VoiceTranscribe`, `VoiceSynthesize`, and `IntentRoute` syscalls to the kernel for capability-secured execution. |
| Voice Interaction | Inference Routing | Voice feeds 18-dimensional feature vectors (14 base + 4 voice: pitch, cadence, urgency, ambient noise) to `TinyDancerRouter` for strategy selection. |
| Phone Runtime | Agent Lifecycle | Phone starts/stops agents via `AgentSpawner`, managing background agent scheduling and foreground activation based on user context. |
| Phone Runtime | Swarm Coordination | Phone joins the swarm as a Zone A-Mobile node, participating in consensus and receiving scatter-gather work units. |
| Agent Marketplace | Phone Runtime | Phone Runtime is downstream of Marketplace; it discovers and installs agent packages from the marketplace catalog. |
| Personal Mesh | Phone Runtime | Mesh registers the phone as a device; Phone Runtime provides device capabilities and battery state for mesh scheduling. |
| Federated Learning | Observation & Health | Federation collects anonymized patterns from SONA's PatternBank; distributes aggregated LoRA updates back to cognitive layer. |
| Subscription Billing | Agent Lifecycle | Billing tier determines agent concurrency limits; Agent Lifecycle enforces these via TierCapabilityToken caveats. |
| Subscription Billing | Federated Learning | Free tier cannot participate in federation. Tier enforcement gates contribution and distribution. |

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
| `VoiceTranscribed` | Voice Interaction | Kernel Syscall (intent decomposition), Observation & Health (session logging) |
| `IntentRouted` | Voice Interaction | Inference Routing (strategy selection with voice features) |
| `VoiceSynthesized` | Voice Interaction | Phone Runtime (audio playback) |
| `AgentInstalled` | Agent Marketplace | Agent Lifecycle (registers new agent type), Phone Runtime (UI update) |
| `AgentPublished` | Agent Marketplace | Agent Marketplace (discovery index update) |
| `PhoneSessionStarted` | Phone Runtime | Voice Interaction (activates pipeline), Swarm Coordination (zone join) |
| `LifeScoreUpdated` | Phone Runtime | Observation & Health (engagement metrics) |

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

rlmx-voice [NEW]          (Voice Interaction context)
  +-- rlmx-kernel
  +-- rlmx-ruvllm          (TinyDancerRouter 18-dim voice features)
  +-- rlmx-cognitive        (session memory via SONA)

rlmx-phone [NEW]          (Phone Runtime context)
  +-- rlmx-kernel
  +-- rlmx-voice
  +-- rlmx-agents
  +-- rlmx-swarm

rlmx-marketplace [NEW]    (Agent Marketplace context)
  +-- rlmx-kernel
  +-- rlmx-agents
  +-- rlmx-rvf              (signed agent packages)

rlmx-mesh [NEW]            (Personal Mesh context)
  +-- rlmx-kernel

rlmx-federation [NEW]      (Federated Learning context)
  +-- rlmx-kernel
  +-- rlmx-cognitive

rlmx-billing [NEW]         (Subscription Billing context)
  +-- rlmx-kernel

rlmx-napi [NEW]            (NAPI-RS binding layer)
  +-- rlmx-kernel

rlmx-wasm [NEW]            (WASM kernel subset)
  +-- rlmx-kernel (feature-gated wasm-bindgen)
```

## Integration Mapping

This section maps the concrete integration points between contexts: which
kernel syscalls each context uses, which domain events flow between contexts,
and which capability tokens are required.

### Syscall Usage by Context

Each context dispatches a specific subset of the 12 kernel syscalls. Contexts
must not invoke syscalls outside their declared set. Capability tokens are
scoped accordingly.

| Context | Syscalls Used | Purpose |
|---------|--------------|---------|
| **Kernel Syscall** | All 12 | Core dispatch; owns all syscall definitions |
| **Agent Lifecycle** | `ProcessFork`, `ProcessSend`, `ProcessRecv`, `HaltCheck` | Agent spawning, inter-agent messaging, graceful shutdown |
| **Swarm Coordination** | `ProcessFork`, `ProcessSend`, `ProcessRecv`, `StateMutate` | Node-level agent management, state replication across zones |
| **Inference Routing** | `VecSearch`, `AttentionSelect` | Router feature lookup, attention mechanism selection |
| **Research & Evolution** | `VecInsert`, `VecSearch`, `GraphQuery`, `GraphDiffuse`, `StateMutate` | Hypothesis storage, knowledge graph exploration, genome mutation |
| **Observation & Health** | `VecInsert`, `VecSearch`, `StateMutate` | Pattern storage in SONA, DAG optimizer state updates |
| **Container & Storage** | `StateMutate` | Witness chain append, container seal/branch operations |
| **Plugin & Domain** | `VecInsert`, `GraphQuery` | Domain data ingestion, domain-specific graph queries |
| **Voice Interaction** | `VecSearch`, `AttentionSelect`, `ProcessSend` | Intent routing, attention-based response selection, fan-out dispatch |
| **Phone Runtime** | `ProcessFork`, `ProcessSend`, `ProcessRecv`, `HaltCheck`, `StateMutate` | Agent lifecycle on device, engagement state persistence, offline queue flush |
| **Agent Marketplace** | `VecSearch`, `StateMutate` | Agent discovery index search, listing state transitions |

### Domain Event Flow Matrix

Events are the primary mechanism for cross-context communication. Each row is
a domain event, each column indicates whether a context produces (P) or
consumes (C) that event.

| Event | Kernel | Agent | Swarm | Inference | Research | Cognitive | RVF | Plugin | Voice | Phone | Marketplace |
|-------|--------|-------|-------|-----------|----------|-----------|-----|--------|-------|-------|-------------|
| `SyscallDispatched` | P | | | | | C | | | | | |
| `AgentSpawned` | | P | C | | | | | | | C | |
| `QueryRouted` | | | | P | | C | | | | | |
| `ExperimentCompleted` | | | | | P | C | | | | | |
| `NodeJoined` | | C | P | | | | | | | | |
| `NodeLeft` | | C | P | | | | | | | | |
| `StateMutated` | P | | | | | | C | | | | |
| `VoiceTranscribed` | C | | | | | C | | | P | | |
| `IntentRouted` | | | | C | | | | | P | | |
| `VoiceSynthesized` | | | | | | | | | P | C | |
| `AgentInstalled` | | C | | | | | | | | C | P |
| `AgentPublished` | | | | | | | | | | | P |
| `PhoneSessionStarted` | | | C | | | | | | C | P | |
| `LifeScoreUpdated` | | | | | | C | | | | P | |
| `AgentSubmitted` | | | | | | | | | | | P |
| `ReviewCompleted` | | | | | | | | | | | P |
| `SavingsRecorded` | | | | | | | | | | P | |
| `SessionStarted` | | | | | | | | | P | C | |
| `SessionEnded` | | | | | | C | | | P | C | |
| `BatteryPolicyChanged` | | | | | | | | | C | P | |
| `NotificationSent` | | | | | | | | | | P | |
| `AgentSuspended` | | C | | | | | | | | P | P |
| `MeshDeviceJoined` | | | C | | | | | | | C | |
| `MeshDeviceLeft` | | | C | | | | | | | C | |
| `MeshSyncCompleted` | | | | | | C | | | | C | |
| `FederationCycleStarted` | | | | | | C | | | | | |
| `FederationCycleCompleted` | | | | | | C | | | | | |
| `SubscriptionChanged` | | | | | | | | | | C | C |
| `TierUpgraded` | | C | | | | | | | | C | C |

### Capability Token Scoping by Context

Each context receives capability tokens scoped to its declared syscall subset.
Tokens are derived from a parent token via `derive_child()` with permission
narrowing. No context can self-escalate beyond its declared scope.

| Context | Token Scope | Derived From | Additional Constraints |
|---------|------------|-------------|----------------------|
| **Kernel Syscall** | `SyscallPermission::All` | Root token | System-level only; never exposed to external consumers |
| **Agent Lifecycle** | `ProcessFork`, `ProcessSend`, `ProcessRecv`, `HaltCheck` | Kernel root | Per-agent tokens further narrowed by `PermissionMatrix` row |
| **Swarm Coordination** | `ProcessFork`, `ProcessSend`, `ProcessRecv`, `StateMutate` | Kernel root | Zone-scoped: tokens carry zone assignment, cannot cross zones without coordinator approval |
| **Inference Routing** | `VecSearch`, `AttentionSelect` | Kernel root | Read-only data access; no mutation permissions |
| **Research & Evolution** | `VecInsert`, `VecSearch`, `GraphQuery`, `GraphDiffuse`, `StateMutate` | Agent Lifecycle (Researcher token) | COW-branched mutations only; cannot mutate main state without Reviewer approval |
| **Observation & Health** | `VecInsert`, `VecSearch`, `StateMutate` | Kernel root | Pattern storage scoped to SONA subsystem; no graph or process permissions |
| **Container & Storage** | `StateMutate` | Kernel root | Scoped to witness chain and RVF container operations only |
| **Plugin & Domain** | `VecInsert`, `GraphQuery` | Agent Lifecycle | Narrowed per-plugin based on `DomainPlugin::safety_constraints()` |
| **Voice Interaction** | `VecSearch`, `AttentionSelect`, `ProcessSend` | Phone Runtime coordinator token | Cannot fork new processes; dispatches intents through existing agents only |
| **Phone Runtime** | `ProcessFork`, `ProcessSend`, `ProcessRecv`, `HaltCheck`, `StateMutate` | Kernel root (device-scoped) | Concurrency-limited by `LightweightCoordinator::max_concurrent` (3-8 based on battery) |
| **Agent Marketplace** | `VecSearch`, `StateMutate` | Kernel root | Discovery index only; no agent execution permissions. `StateMutate` scoped to listing state transitions |

### Cross-Context ACL Summary

| Boundary | ACL Mechanism | What Crosses | What Is Blocked |
|----------|--------------|-------------|-----------------|
| Voice -> Kernel | Intent translation | `Intent` structs as syscall params | Raw audio, speaker embeddings |
| Voice -> Inference | Feature normalization | 18-dim float vector (14 base + 4 voice) | Raw audio features, unnormalized values |
| Voice -> Phone | Event bridge | `VoiceSynthesized` event with `OpusChunk` refs | TTS model internals, SSML markup |
| Voice -> Cognitive | SONA PatternBank API | `(query_embedding, actions, quality)` triples | Session details, turn-level data, speaker identity |
| Phone -> Kernel | Syscall dispatch | Capability-scoped syscalls | Device-specific OS handles, raw battery readings |
| Phone -> Swarm | Zone join protocol | `DeviceCapabilities` summary, `NodeId` | OS-level process handles, notification state |
| Phone -> Agents | AgentSpawner API | `AgentType`, scoped `CapabilityToken` | Battery policy internals, widget state |
| Marketplace -> RVF | Container API | Signed RVF containers, content hashes | Billing data, publisher credentials |
| Marketplace -> Agents | Published agent packages | `AgentType` definition, permission declarations | Runtime state, execution context |
| Marketplace -> Phone | Install event | `AgentInstalled` event with agent metadata | Pricing details, publisher identity, review status |
| Cognitive -> RVF | WitnessChain conformist | Audit entries in `WitnessChain` format | SONA internals, Fisher information matrices |
