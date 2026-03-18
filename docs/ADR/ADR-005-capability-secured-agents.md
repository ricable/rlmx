# ADR-005: Capability-Secured Agent Types with Permission Matrix

## Status
Proposed

## Date
2026-03-18

## Context

The RLMX kernel provides two primitives relevant to agent security:

1. **CapabilityToken** (`crates/rlmx-kernel/src/capability.rs`): HMAC-SHA256-signed tokens with per-syscall permission grants (`granted_syscalls: Vec<SyscallPermission>`), TTL expiry, and hierarchical derivation. Children cannot exceed parent permissions.

2. **ProcessFork** (`crates/rlmx-kernel/src/syscall.rs`): The `Syscall::ProcessFork` variant creates child processes with narrowed capability tokens via the process subsystem (`crates/rlmx-kernel/src/process.rs`).

Currently, these primitives are untyped -- any process can be forked with any subset of permissions, and there is no concept of agent roles or specialization. In the 25-node swarm (ADR-001), this creates problems:

- No way to enforce that a monitoring agent cannot perform `StateMutate` operations
- No role-based placement: the `PlacementPolicy` cannot map agent types to zones
- No audit trail of which agent type performed which syscall
- The MCP RBAC model (`crates/rlmx-mcp/src/server.rs`) operates at the transport layer (Viewer/Operator/Engineer/Admin/Auditor/System) but has no kernel-level analog for agent processes

The swarm needs 12 specialized agent types spanning coordination, inference, analysis, and validation roles. Each type requires a well-defined permission envelope.

## Decision

Implement **12 typed agent roles** with a **12x12 permission matrix** mapping agent types to the 12 `SyscallPermission` variants. Introduce a new crate `rlmx-agents` that defines agent types, their permission envelopes, and the spawn protocol.

### Agent Type Hierarchy

```
Coordinator (PID 0, root of capability tree)
  |
  +-- Router (routes queries to strategies/zones)
  |
  +-- Worker (executes inference via Strategy dispatch)
  |
  +-- Monitor (health checks, gossip participation)
  |
  +-- Replicator (consensus log replication)
  |
  +-- Embedder (vector operations, embedding generation)
  |
  +-- Validator (output validation, safety checks)
  |
  +-- Reviewer (code review, quality assessment)
  |
  +-- Analyst (graph queries, pattern analysis)
  |
  +-- Researcher (knowledge retrieval, VecSearch)
  |     |
  |     +-- Experimenter (hypothesis testing, scoped mutations)
  |           |
  |           +-- sub-Experimenter (narrowed scope, read-only)
  |
  +-- Trainer (SONA adaptation, DagOptimizer updates)
```

### Permission Matrix

Each cell is Allow (A) or Deny (-). The 12 `SyscallPermission` variants from `crates/rlmx-kernel/src/types.rs`:

```
Agent Type       | VecI VecS VecD GrQ  GrC  GrD  PFrk PSnd PRcv StMu AtSl Halt
-----------------+---------------------------------------------------------------
Coordinator      |  A    A    A    A    A    A    A    A    A    A    A    A
Router           |  -    A    -    -    -    -    -    A    A    -    A    A
Worker           |  A    A    -    A    -    -    -    A    A    -    -    A
Monitor          |  -    A    -    -    -    -    -    A    A    -    -    A
Replicator       |  -    -    -    -    -    -    -    A    A    A    -    -
Embedder         |  A    A    A    -    -    -    -    A    A    -    -    -
Validator        |  -    A    -    A    -    -    -    A    A    -    A    A
Reviewer         |  -    A    -    A    A    -    -    A    A    -    A    -
Analyst          |  -    A    -    A    A    A    -    A    A    -    A    -
Researcher       |  -    A    -    A    -    -    A    A    A    -    -    -
Experimenter     |  A    A    -    A    -    A    -    A    A    A    -    A
Trainer          |  A    A    A    -    -    -    -    A    A    A    -    -
```

Key: VecI=VecInsert, VecS=VecSearch, VecD=VecDelete, GrQ=GraphQuery, GrC=GraphCut, GrD=GraphDiffuse, PFrk=ProcessFork, PSnd=ProcessSend, PRcv=ProcessRecv, StMu=StateMutate, AtSl=AttentionSelect, Halt=HaltCheck.

Design principles:
- Only `Coordinator` has `ProcessFork` (plus `Researcher` for spawning Experimenters)
- `ProcessSend`/`ProcessRecv` granted to all types (inter-agent communication is fundamental)
- `StateMutate` restricted to `Coordinator`, `Replicator`, `Experimenter`, and `Trainer`
- `VecDelete` restricted to `Coordinator`, `Embedder`, and `Trainer` (destructive operation)
- `Router` has minimal permissions: only reads (VecSearch) and control flow (AttentionSelect, HaltCheck)

### New Crate: `rlmx-agents`

19 files: `Cargo.toml` + `src/{lib, agent_type, permission, spawn, coordinator, router_agent, worker, monitor, replicator, embedder, validator, reviewer, analyst, researcher, experimenter, trainer, lifecycle, error}.rs`. Agent state machine: `Init -> Running -> Paused -> Terminated`. Dependencies: `rlmx-kernel` (for `Syscall`, `CapabilityToken`, `ProcessId`, `SyscallPermission`).

### Core Types

```rust
// crates/rlmx-agents/src/agent_type.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    Coordinator,
    Router,
    Worker,
    Monitor,
    Replicator,
    Embedder,
    Validator,
    Reviewer,
    Analyst,
    Researcher,
    Experimenter,
    Trainer,
}

// crates/rlmx-agents/src/permission.rs
pub struct PermissionMatrix {
    matrix: [[bool; 12]; 12],  // [agent_type_index][syscall_permission_index]
}

impl PermissionMatrix {
    /// Returns the allowed SyscallPermissions for a given agent type.
    pub fn permissions_for(&self, agent_type: AgentType) -> Vec<SyscallPermission> { ... }

    /// Validates that a requested permission set does not exceed the agent type's envelope.
    pub fn validate(&self, agent_type: AgentType, requested: &[SyscallPermission]) -> bool { ... }
}

// crates/rlmx-agents/src/spawn.rs -- summarized for brevity
pub struct AgentSpawner { kernel: Arc<KernelContext>, matrix: PermissionMatrix, secret: Vec<u8> }

// spawn() method: (1) looks up allowed permissions from the matrix via permissions_for(),
// (2) verifies parent token contains all required permissions (hierarchical narrowing rule),
// (3) derives a narrowed CapabilityToken via CapabilityToken::derive(),
// (4) issues Syscall::ProcessFork to create the child process.
// Returns (ProcessId, CapabilityToken) or AgentError::InsufficientParentPermissions.
```

### Zone-to-Agent Mapping

Extending ADR-001's `PlacementPolicy`, agent types have zone affinity:

| Agent Type | Primary Zone | Agent Type | Primary Zone |
|-----------|-------------|-----------|-------------|
| Coordinator | A (PBFT) | Embedder | A, B |
| Router | A, B | Validator | A |
| Worker | A, B, D | Analyst | A, B |
| Monitor | B, C | Researcher | B, C |
| Replicator | A, B | Experimenter | A (PBFT) |
| Reviewer | A | Trainer | A (GPU) |

### Phase 2: ruvix-cap Migration

Phase 2 replaces HMAC-SHA256 `CapabilityToken` with `ruvix-cap`: Ed25519 signatures (no shared secret), Macaroon-style caveat attenuation, time-bounded delegation chains, and revocation certificates. Migration via `TryFrom<RuvixCap>` trait on `CapabilityToken` -- `PermissionMatrix` and `AgentSpawner` remain unchanged.

## Consequences

### Positive
- Principle of least privilege enforced at the kernel level for all agent processes
- Permission violations are caught at `dispatch()` time, before any syscall executes
- Typed agents enable zone-aware placement (ADR-001) and workload-specific monitoring
- Hierarchical derivation prevents privilege escalation: a Researcher cannot spawn an Experimenter with StateMutate unless the Researcher itself has StateMutate (it does not)

### Negative
- 12 agent types with 19 source files increases crate count from 9 to 10
- Permission matrix must be updated when new syscall variants are added to the kernel
- Agent type rigidity: adding a 13th type requires matrix expansion and recompilation

### Risks
- Over-restrictive permissions may block legitimate operations, requiring matrix updates
- `ProcessFork` restriction to Coordinator and Researcher creates a bottleneck for dynamic agent spawning under load
- Phase 2 ruvix-cap migration requires updating all token validation paths across `rlmx-kernel`, `rlmx-mcp`, and `rlmx-agents`

## Alternatives Considered

1. **Flat RBAC (MCP-style)**: Reuse the 6-role model (Viewer/Operator/Engineer/Admin/Auditor/System). Rejected -- MCP roles are transport-layer constructs for HTTP/stdio sessions, not kernel processes.

2. **No agent typing**: Any process holds any SyscallPermission subset. Rejected -- no structural guarantee about agent behavior; audit log shows ProcessId but not intended role.

3. **External policy engine (OPA/Cedar)**: Rejected -- adds network latency to every syscall dispatch (hot path) and cannot enforce hierarchical derivation without duplicating kernel state.

4. **seL4-style formal verification**: Rejected as disproportionate. Phase 2 ruvix-cap moves toward stronger guarantees incrementally.

## References
- `crates/rlmx-kernel/src/capability.rs` -- `CapabilityToken`, HMAC-SHA256 signing, hierarchical derivation
- `crates/rlmx-kernel/src/types.rs` -- `SyscallPermission` enum (12 variants + All), `Capability` struct
- `crates/rlmx-kernel/src/syscall.rs` -- `Syscall::ProcessFork`, 12-variant dispatch
- `crates/rlmx-kernel/src/process.rs` -- process subsystem, child process creation
- `crates/rlmx-mcp/src/server.rs` -- RBAC role model (6 roles), `token_roles` map
- `crates/rlmx-rvf/` -- Ed25519 signatures for sealed containers (Phase 2 alignment)
- `crates/rlmx-cognitive/` -- SONA adaptation and DagOptimizer used by Trainer agent
- ADR-001 -- zone definitions, `PlacementPolicy` for zone-to-agent mapping
- ADR-003 -- `TinyDancerRouter` wrapped by Router agent type
