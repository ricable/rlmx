# DDD-002: Kernel Syscall Bounded Context

## Overview

The Kernel Syscall context is the **core domain** of RLMX. It provides the
12-syscall dispatch interface, capability-secured process model, vector memory,
property graph, and proof-gated state mutation. All other contexts interact
with the kernel through this boundary.

**Crate**: `crates/rlmx-kernel/`

## Aggregate Root: KernelContext

Defined in `crates/rlmx-kernel/src/syscall.rs`:

```rust
pub struct KernelContext {
    pub memory: Arc<Mutex<MemoryRegion>>,
    pub graph: Arc<Mutex<Graph>>,
    pub process_manager: Arc<Mutex<ProcessManager>>,
    pub proof_engine: Arc<Mutex<ProofEngine>>,
    pub capability_manager: Arc<Mutex<CapabilityManager>>,
    pub caller_pid: Option<ProcessId>,
}
```

`KernelContext` is the aggregate root because it enforces the consistency
boundary for all subsystems. No subsystem can be accessed without going
through the context, and all access is serialized via `Arc<Mutex<T>>`.

## Entities

### Syscall (12 variants)

Defined in `crates/rlmx-kernel/src/syscall.rs` as `enum Syscall`:

| Family | Variants | Subsystem |
|--------|----------|-----------|
| **Vector Memory** | `VecInsert`, `VecSearch`, `VecDelete` | `MemoryRegion` |
| **Graph** | `GraphQuery`, `GraphCut`, `GraphDiffuse` | `Graph` |
| **Process** | `ProcessFork`, `ProcessSend`, `ProcessRecv` | `ProcessManager` |
| **Proof** | `StateMutate` | `ProofEngine` |
| **Cognitive** | `AttentionSelect`, `HaltCheck` | Inline logic |

Each variant has identity through its family name (returned by `Syscall::family()`),
used for capability checking.

### CapabilityToken

Defined in `crates/rlmx-kernel/src/capability.rs`:

- **Identity**: `id: Uuid`
- **Owner**: `owner: ProcessId`
- **Granted syscalls**: `granted_syscalls: Vec<SyscallPermission>`
- **Scope**: `scope: String` (memory isolation boundary)
- **Expiry**: `expiry: DateTime<Utc>`
- **Signature**: `issuer_signature: String` (HMAC-SHA256, hex-encoded)

### Process

Defined in `crates/rlmx-kernel/src/process.rs`:

- **Identity**: `id: ProcessId` (UUID)
- **Parent**: `parent: Option<ProcessId>` (fork hierarchy)
- **Capability token**: Scoped to this process
- **Memory scope**: String determining which memory region the process can access
- **Status**: `ProcessStatus` enum (`Running`, `Waiting`, `Completed`, `Failed`)
- **Channel**: `mpsc::Sender<KernelMessage>` / `mpsc::Receiver<KernelMessage>`

### VectorMemory (MemoryRegion)

Defined in `crates/rlmx-kernel/src/memory.rs`:

- Brute-force cosine similarity with 64-dim hash-based pseudo-embeddings
- 3-tier classification: `Hot` / `Warm` / `Cold` (based on access frequency)
- Each segment: `ContextSegment { id, embedding, content, metadata, tier, access_count, ... }`

### Graph

Defined in `crates/rlmx-kernel/src/graph.rs`:

- In-memory property graph with nodes and weighted edges
- Minimal Cypher parser for `MATCH (n) WHERE ... RETURN ...` patterns
- Min-cut: Karger randomized and Stoer-Wagner deterministic algorithms
- Heat-kernel diffusion over graph edges

### ProofChain (WitnessChain + ProofEngine)

Defined in `crates/rlmx-kernel/src/proof.rs`:

- `WitnessChain`: SHA-256-chained append-only audit log
- Each `Witness` has: `id`, `action_hash`, `reasoning_chain_hash`, `evidence_refs`, `prev_hash`, `content_hash`
- `ProofEngine::validate()` appends a witness and returns a `Proof` with `valid: bool`

## Value Objects

| Value Object | Location | Description |
|-------------|----------|-------------|
| `SyscallPermission` | `types.rs` | Enum with 12 variants + `All`. Compared by value, no identity. |
| `ProcessId` | `types.rs` | Type alias for `Uuid`. Identifies a process. |
| `Capability` | `types.rs` | `{ name, permissions: Vec<SyscallPermission> }` |
| `SegmentMetadata` | `types.rs` | `{ source, plugin, segment_type, extra }` |
| `SearchFilters` | `types.rs` | `{ source, plugin, segment_type, tier, after, before }` |
| `SegmentTier` | `types.rs` | Enum: `Hot`, `Warm`, `Cold` |
| `MinCutAlgorithm` | `types.rs` | Enum: `Karger`, `StoerWagner` |
| `ProofRequest` | `types.rs` | `{ confidence_threshold, require_evidence }` |
| `SearchHit` | `types.rs` | `{ segment_id, content, score, metadata }` |
| `KernelMessage` | `types.rs` | `{ from, to, payload: Value, timestamp }` |
| `WitnessHash` | `proof.rs` | The `content_hash` field on `Witness` (SHA-256 hex string) |

## Domain Events

| Event | Trigger | Data |
|-------|---------|------|
| `SyscallDispatched` | `dispatch()` completes | `{ syscall_family, caller_pid, result_variant, timestamp }` |
| `ProcessForked` | `ProcessFork` syscall | `{ parent_pid, child_pid, capabilities, memory_scope }` |
| `StateMutated` | `StateMutate` syscall | `{ witness_id, action, valid, confidence }` |
| `CapabilityDerived` | `derive_child_token()` | `{ parent_token_id, child_token_id, child_permissions }` |
| `CapabilityRevoked` | `revoke()` | `{ token_id, revoked_at }` |
| `SegmentTierChanged` | Access-frequency reclassification | `{ segment_id, old_tier, new_tier }` |

## Repositories

| Repository | Struct | Location | Purpose |
|-----------|--------|----------|---------|
| Process Registry | `ProcessManager` | `process.rs` | `fork()`, `kill()`, `reap()`, `complete()`, `list()`, `get()` |
| Vector Store | `MemoryRegion` | `memory.rs` | `insert()`, `search()`, `delete()` |
| Graph Store | `Graph` | `graph.rs` | `add_node()`, `add_edge()`, `cypher_query()`, `min_cut()`, `diffuse()` |
| Token Store | `CapabilityManager` | `capability.rs` | `create_token()`, `validate()`, `revoke()`, `derive_child_token()` |
| Proof Store | `ProofEngine` | `proof.rs` | `validate()` (appends + returns proof) |

## Domain Services

### `dispatch(syscall, ctx) -> KernelResult<SyscallResult>`

The central routing function in `syscall.rs`. Pattern-matches on the `Syscall`
enum and delegates to the appropriate subsystem via the `KernelContext`. Returns
one of 12 `SyscallResult` variants.

### `CapabilityManager::derive_child_token(parent_id, child_owner, permissions, scope, ttl)`

Creates a child token that is a strict subset of the parent's permissions.
Enforces: child cannot exceed parent permissions, child expiry cannot exceed
parent expiry, revoked parents cannot produce children.

### `ProofEngine::validate(action, reasoning, evidence, confidence, request)`

Records a state mutation in the witness chain and produces a `Proof`. The proof
is valid only if confidence meets the threshold and evidence requirements are
satisfied.

### `WitnessChain::verify_integrity()`

Walks the entire chain verifying: (1) genesis has empty `prev_hash`, (2) each
entry's `prev_hash` equals previous entry's `content_hash`, (3) each
`content_hash` is consistent with its fields.

## Invariants

1. **Capability subset rule**: A child token's `granted_syscalls` must be a
   subset of its parent's. Enforced in `derive_child_token()`.

2. **Capability expiry rule**: A child token's expiry cannot exceed the parent's
   expiry. Enforced via `std::cmp::min(child_ttl, parent.expiry)`.

3. **Proof-gated mutation**: Every `StateMutate` call appends to the witness
   chain. No state mutation can bypass `ProofEngine::validate()`.

4. **Signature integrity**: All `CapabilityToken` instances carry an
   HMAC-SHA256 signature. `validate()` verifies the signature before granting
   access. Constant-time comparison via `hmac::verify_slice`.

5. **Process isolation**: Each process has a `memory_scope` string. Processes
   can only access memory within their scope (enforced at the application layer).

6. **Channel bounds**: Process channels use `mpsc::channel(256)`. Backpressure
   applies when a process's inbox is full.

## File Map

| File | Types |
|------|-------|
| `crates/rlmx-kernel/src/syscall.rs` | `Syscall`, `KernelContext`, `dispatch()` |
| `crates/rlmx-kernel/src/capability.rs` | `CapabilityToken`, `CapabilityManager` |
| `crates/rlmx-kernel/src/process.rs` | `Process`, `ProcessManager`, `ProcessStatus` |
| `crates/rlmx-kernel/src/memory.rs` | `MemoryRegion`, `ContextSegment`, `cosine_similarity()` |
| `crates/rlmx-kernel/src/graph.rs` | `Graph`, Cypher parser, min-cut, diffusion |
| `crates/rlmx-kernel/src/proof.rs` | `ProofEngine`, `WitnessChain`, `Witness`, `Proof` |
| `crates/rlmx-kernel/src/types.rs` | All shared types, `KernelError` |
| `crates/rlmx-kernel/src/scheduler.rs` | `Strategy`, `Scheduler`, `SchedulerConfig` |
| `crates/rlmx-kernel/src/lib.rs` | Public re-exports |
