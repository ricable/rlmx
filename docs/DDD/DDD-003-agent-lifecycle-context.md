# DDD-003: Agent Lifecycle Bounded Context

## Overview

The Agent Lifecycle context manages the 14 specialized agent types that compose
the RLMX distributed swarm. It handles agent creation (via kernel `ProcessFork`),
termination, capability scoping, and the permission matrix governing which
agents can communicate with which.

**Crate**: `crates/rlmx-agents/` (planned)

## Aggregate Root: AgentRegistry

```rust
pub struct AgentRegistry {
    agents: HashMap<AgentId, Agent>,
    spawner: AgentSpawner,
    permission_matrix: PermissionMatrix,
    limits: AgentLimits,
}
```

The `AgentRegistry` is the single point of truth for all live agents. It
enforces per-type concurrency limits, validates permission matrices before
allowing communication, and delegates to `AgentSpawner` for creation.

## Entities

### Agent (12 types)

Each agent is a kernel `Process` with additional domain semantics:

| # | AgentType | Role | Capability Scope | Max Concurrent |
|---|-----------|------|-----------------|----------------|
| 1 | `Coordinator` | Orchestrates task decomposition and agent assignment | `All` (scoped to swarm) | 1 per zone |
| 2 | `Researcher` | Generates hypotheses, designs experiments | `VecSearch`, `GraphQuery`, `ProcessFork`, `ProcessSend` | 3 |
| 3 | `Router` | Routes queries to optimal strategy/tier | `VecSearch`, `AttentionSelect`, `HaltCheck` | 2 |
| 4 | `Experimenter` | Executes experiments with COW-branched state | `VecInsert`, `VecSearch`, `StateMutate`, `ProcessSend` | 8 |
| 5 | `Worker` | Executes concrete tasks (code gen, analysis) | `VecInsert`, `VecSearch`, `GraphQuery`, `ProcessSend` | 10 |
| 6 | `Monitor` | Health checks, metrics, anomaly detection | `VecSearch`, `GraphQuery`, `ProcessRecv` | 2 |
| 7 | `Reviewer` | Validates outputs, enforces quality gates | `VecSearch`, `GraphQuery`, `HaltCheck` | 3 |
| 8 | `Trainer` | Online learning, SONA adaptation, LoRA updates | `VecSearch`, `StateMutate`, `AttentionSelect` | 2 |
| 9 | `Validator` | Proof validation, witness chain auditing | `VecSearch`, `StateMutate`, `HaltCheck` | 2 |
| 10 | `Replicator` | State replication across nodes | `VecInsert`, `VecSearch`, `ProcessSend`, `ProcessRecv` | 3 |
| 11 | `Embedder` | Generates and indexes embeddings | `VecInsert`, `VecDelete`, `ProcessRecv` | 4 |
| 12 | `Analyst` | Synthesizes results, generates reports | `VecSearch`, `GraphQuery`, `GraphDiffuse` | 3 |
| 13 | `VoiceCoordinator` | Manages voice pipeline and session lifecycle | `VecSearch`, `ProcessSend`, `ProcessRecv`, `AttentionSelect` | 2 |
| 14 | `MarketplaceManager` | Handles agent publishing, review, and billing | `VecSearch`, `GraphQuery`, `ProcessSend` | 2 |

```rust
pub struct Agent {
    pub id: AgentId,
    pub agent_type: AgentType,
    pub process_id: ProcessId,       // kernel process backing this agent
    pub capability_token_id: Uuid,   // scoped token from CapabilityManager
    pub zone: ZoneAssignment,
    pub model_tier: ModelTier,
    pub status: AgentStatus,
    pub config: AgentConfig,
    pub spawned_at: DateTime<Utc>,
    pub parent_agent: Option<AgentId>,
}
```

### AgentConfig

```rust
pub struct AgentConfig {
    pub max_memory_mb: usize,
    pub inference_timeout: Duration,
    pub retry_policy: RetryPolicy,
    pub escalation_threshold: f64,
}
```

### ProcessGroup

A logical grouping of agents working on a shared task:

```rust
pub struct ProcessGroup {
    pub id: Uuid,
    pub coordinator: AgentId,
    pub members: Vec<AgentId>,
    pub shared_memory_scope: String,
    pub created_at: DateTime<Utc>,
}
```

## Value Objects

| Value Object | Definition |
|-------------|------------|
| `AgentType` | Enum with 14 variants: `Coordinator`, `Researcher`, `Router`, `Experimenter`, `Worker`, `Monitor`, `Reviewer`, `Trainer`, `Validator`, `Replicator`, `Embedder`, `Analyst`, `VoiceCoordinator`, `MarketplaceManager` |
| `AgentId` | Newtype wrapping `Uuid`. Distinguished from `ProcessId`. |
| `AgentStatus` | Enum: `Spawning`, `Ready`, `Busy`, `Paused`, `Terminating`, `Terminated` |
| `PermissionMatrix` | 15x14 boolean matrix. `matrix[sender_type][receiver_type]` = can send. |
| `ZoneAssignment` | Enum: `ZoneA`, `ZoneB`, `ZoneC`, `Cloud`, `Browser` |
| `ModelTier` | Enum: `Small` (Haiku-class), `Medium` (Sonnet-class), `Large` (Opus-class), `Edge` (local GGUF), `Custom(String)` |

### Permission Matrix (15x14)

```
           Coord Resrch Router Exper Worker Monit Review Train Valid Repli Embed Analys
Coord        -     W      W     W     W     R     R      W     R    W     W     R
Researcher   R     -      R     W     -     -     -      -     -    -     -     -
Router       R     -      -     -     W     -     -      -     -    -     -     -
Experimenter R     R      -     -     -     -     -      W     -    -     W     -
Worker       R     -      -     -     -     -     W      -     -    -     -     W
Monitor      W     -      -     -     -     -     -      -     -    -     -     -
Reviewer     R     -      -     -     R     -     -      -     W    -     -     -
Trainer      R     -      -     -     -     -     -      -     -    -     -     -
Validator    R     -      -     -     -     W     -      -     -    -     -     -
Replicator   R     -      -     -     -     -     -      -     -    -     -     -
Embedder     R     -      -     -     -     -     -      -     -    R     -     -
Analyst      R     -      -     -     -     -     -      -     -    -     -     -

W = Write (ProcessSend), R = Read (ProcessRecv), - = denied
```

## Domain Events

| Event | Trigger | Data |
|-------|---------|------|
| `AgentSpawned` | `spawn_agent()` | `{ agent_id, agent_type, process_id, zone, model_tier }` |
| `AgentTerminated` | `terminate_agent()` | `{ agent_id, reason, lifetime_ms }` |
| `AgentEscalated` | Confidence below threshold | `{ agent_id, from_tier, to_tier, query_hash }` |
| `PermissionViolated` | `ProcessSend` to unauthorized target | `{ sender_id, target_id, sender_type, target_type }` |
| `AgentStatusChanged` | Any status transition | `{ agent_id, old_status, new_status }` |
| `ProcessGroupCreated` | Coordinator assembles team | `{ group_id, coordinator_id, member_ids }` |

## Factories

### AgentSpawner

```rust
pub struct AgentSpawner {
    kernel_ctx: Arc<KernelContext>,
}

impl AgentSpawner {
    /// Spawn a new agent by:
    /// 1. Looking up the required SyscallPermissions for the AgentType
    /// 2. Deriving a child CapabilityToken from the parent (Coordinator) token
    /// 3. Issuing ProcessFork syscall with the scoped token
    /// 4. Registering the Agent in the AgentRegistry
    pub async fn spawn(
        &self,
        agent_type: AgentType,
        zone: ZoneAssignment,
        model_tier: ModelTier,
        parent_token_id: Uuid,
        config: AgentConfig,
    ) -> Result<Agent, AgentError>;
}
```

The spawner maps `AgentType` to the required `Vec<SyscallPermission>` using
a lookup table, then calls `CapabilityManager::derive_child_token()` to create
a token that is a strict subset of the parent's.

## Domain Services

### `spawn_agent(agent_type, zone, model_tier, config) -> Result<AgentId>`

1. Check concurrency limits for the requested `AgentType`.
2. Select parent token (Coordinator's token, or root token for Coordinators).
3. Derive scoped child token via `CapabilityManager::derive_child_token()`.
4. Execute `ProcessFork` syscall to create the kernel process.
5. Register the `Agent` in `AgentRegistry`.
6. Emit `AgentSpawned` event.

### `terminate_agent(agent_id, reason) -> Result<()>`

1. Look up agent in registry.
2. Send termination signal via `ProcessSend`.
3. Wait for graceful shutdown (with timeout).
4. Call `ProcessManager::kill()` if timeout exceeded.
5. Revoke the agent's capability token.
6. Remove from registry.
7. Emit `AgentTerminated` event.

### `validate_communication(sender_id, target_id) -> Result<()>`

Looks up both agents' types and checks the `PermissionMatrix`. Returns
`PermissionViolated` error if the sender type cannot communicate with the
target type.

## Invariants

1. **Concurrency limits**: Each `AgentType` has a maximum concurrent count.
   `spawn_agent()` checks the registry before creating.

2. **Agent hierarchy**: Coordinators are spawned first (1 per zone). All other
   agents are spawned by a Coordinator and receive tokens derived from the
   Coordinator's token.

3. **Zone placement**: An agent's `ZoneAssignment` constrains which
   `SwarmNode` it can run on. Zone A agents cannot be placed on Zone C nodes.

4. **Capability descent**: Agent tokens are always derived from a parent token.
   A Worker's token is a subset of its Coordinator's token, which is a subset
   of the root system token. No agent can escalate beyond its Coordinator.

5. **Permission matrix enforcement**: All `ProcessSend` operations between
   agents are validated against the 15x14 matrix before dispatch.

## Agent Spawn Tree (PID hierarchy)

```
System Root Token (SyscallPermission::All)
  |
  +-- Coordinator (Zone A)  [PID: 001]
  |     +-- Router           [PID: 002]  (derived from 001's token)
  |     +-- Worker           [PID: 003]
  |     +-- Worker           [PID: 004]
  |     +-- Researcher       [PID: 005]
  |     |     +-- Experimenter [PID: 006]  (derived from 005's token)
  |     |     +-- Experimenter [PID: 007]
  |     +-- Monitor          [PID: 008]
  |     +-- Embedder         [PID: 009]
  |
  +-- Coordinator (Zone B)  [PID: 010]
  |     +-- ...
  |
  +-- Coordinator (Cloud)   [PID: 020]
        +-- Trainer          [PID: 021]
        +-- Analyst          [PID: 022]
        +-- Reviewer         [PID: 023]
```

## File Map (planned)

| File | Types |
|------|-------|
| `crates/rlmx-agents/src/types.rs` | `AgentType`, `AgentId`, `AgentStatus`, `ModelTier`, `ZoneAssignment` |
| `crates/rlmx-agents/src/registry.rs` | `AgentRegistry`, `AgentLimits`, `PermissionMatrix` |
| `crates/rlmx-agents/src/spawn.rs` | `AgentSpawner`, spawn logic |
| `crates/rlmx-agents/src/agent.rs` | `Agent`, `AgentConfig`, `ProcessGroup` |
| `crates/rlmx-agents/src/coordinator.rs` | Coordinator-specific orchestration |
| `crates/rlmx-agents/src/researcher.rs` | Research agent behavior |
| `crates/rlmx-agents/src/router.rs` | Routing agent (wraps TinyDancer) |
| `crates/rlmx-agents/src/experimenter.rs` | Experiment execution logic |
| `crates/rlmx-agents/src/worker.rs` | General task execution |
| `crates/rlmx-agents/src/monitor.rs` | Health monitoring agent |
| `crates/rlmx-agents/src/reviewer.rs` | Quality gate validation |
| `crates/rlmx-agents/src/trainer.rs` | Online learning agent |
| `crates/rlmx-agents/src/validator.rs` | Proof and witness validation |
| `crates/rlmx-agents/src/replicator.rs` | State replication agent |
| `crates/rlmx-agents/src/embedder.rs` | Embedding generation agent |
| `crates/rlmx-agents/src/analyst.rs` | Synthesis and reporting agent |
| `crates/rlmx-agents/src/lib.rs` | Public re-exports |
