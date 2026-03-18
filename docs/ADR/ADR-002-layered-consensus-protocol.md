# ADR-002: Layered Consensus Protocol

## Status
Proposed

## Date
2026-03-18

## Context

The 25-node distributed swarm (ADR-001) handles three categories of state with fundamentally different consistency requirements:

1. **Critical state mutations**: `Syscall::StateMutate` writes to the SHA-256 witness-chained audit log (`crates/rlmx-kernel/src/proof.rs`). These require Byzantine fault tolerance -- a compromised RPi4 on an untrusted network must not corrupt the audit chain. The existing `StateMutated { witness_id, success }` result assumes single-node execution.

2. **Metadata and configuration**: Cluster membership, `PlacementPolicy` updates, `CapabilityToken` revocation lists, `SchedulerConfig` changes. These need strong consistency but do not face Byzantine threats (all nodes are operator-controlled).

3. **Health and metrics**: Node load, memory pressure, inference latency histograms, `HealthState` from `SwarmNode`. These tolerate eventual consistency and stale reads (seconds-old data is acceptable for placement decisions).

Running a single consensus protocol across all 25 nodes is impractical:
- PBFT requires 3f+1 nodes and O(n^2) message complexity -- at 25 nodes, that is 625 messages per round
- Raft is efficient but provides only crash fault tolerance, not Byzantine tolerance
- Pure gossip provides no ordering guarantees needed for audit log integrity

RPi4 nodes in Zone C have 4 GB RAM and limited network bandwidth. They cannot participate in PBFT rounds without degrading inference performance on co-located `VecSearch` operations.

## Decision

Implement a **layered consensus stack** with three protocols, each mapped to a zone and data category.

### Layer 1: PBFT for Critical State (Zone A)

The 5 Zone A nodes (3 Mac Studio + 2 NUC-GPU) run PBFT for `StateMutate` syscalls. With f=1 (tolerates 1 Byzantine node), the quorum is 4 nodes. Message complexity is O(25) per round (5^2), acceptable for the ~10 state mutations per second expected workload.

```rust
// crates/rlmx-swarm/src/consensus.rs
pub struct PbftLayer {
    pub view: u64,
    pub leader: NodeId,                    // Zone A Mac Studio, rotates on view change
    pub replicas: Vec<NodeId>,             // all 5 Zone A nodes
    pub log: Vec<PbftEntry>,              // maps to StateMutate witness chain
    pub checkpoint_interval: u64,          // every 100 entries
    pub prepare_quorum: usize,             // 2f+1 = 3
    pub commit_quorum: usize,              // 2f+1 = 3
}

pub struct PbftEntry {
    pub sequence: u64,
    pub syscall: Syscall,                  // always StateMutate variant
    pub witness_id: Uuid,                  // links to proof.rs witness chain
    pub digest: [u8; 32],                  // SHA-256 of serialized syscall
    pub prepares: HashSet<NodeId>,
    pub commits: HashSet<NodeId>,
}
```

### Layer 2: Raft for Metadata (Zone A + Zone B)

Zone A nodes plus all 10 Zone B nodes (8 RPi5 + 2 NUC-CPU) form a 15-node Raft cluster for metadata consensus. The leader is elected from Zone A (preferring NUC-GPU nodes to avoid burdening Mac Studios with Raft log replication during heavy inference). Zone B RPi5 nodes serve as voters, providing read replicas for `SchedulerConfig` and `PlacementPolicy`.

```rust
pub struct RaftLayer {
    pub term: u64,
    pub leader: Option<NodeId>,
    pub voters: Vec<NodeId>,              // 15 nodes (Zone A + Zone B)
    pub log: RaftLog,                     // metadata entries
    pub commit_index: u64,
    pub election_timeout: Duration,       // 150-300ms randomized
    pub heartbeat_interval: Duration,     // 50ms
}

pub enum RaftEntry {
    ClusterMembership(MembershipChange),
    PlacementPolicyUpdate(PlacementPolicy),
    TokenRevocation(Uuid),                // CapabilityToken.id
    ConfigChange(SchedulerConfig),
    ZoneReassignment { node: NodeId, from: Zone, to: Zone },
}
```

Uses the `ruvector-raft` crate for the core Raft state machine, with `rlmx-swarm` providing the transport layer (gRPC via `tonic`) and storage backend (append-only log backed by `rlmx-rvf` sealed containers).

### Layer 3: Gossip for Health/Metrics (All Zones)

All 25 nodes participate in SWIM-style protocol for failure detection and metrics dissemination. Each node gossips to 3 random peers every 500ms. Convergence time for 25 nodes: ~5 seconds (O(log n) rounds).

```rust
pub struct GossipLayer {
    pub members: HashMap<NodeId, MemberState>,
    pub fanout: usize,                     // 3 peers per round
    pub interval: Duration,                // 500ms
    pub suspicion_timeout: Duration,       // 3s before marking suspect
    pub dead_timeout: Duration,            // 10s before marking dead
    pub metrics_buffer: Vec<MetricsUpdate>,
}

pub struct MetricsUpdate {
    pub node: NodeId,
    pub timestamp: DateTime<Utc>,
    pub load: f32,                         // 0.0-1.0
    pub memory_pressure: f32,
    pub inference_latency_p99: Duration,
    pub active_processes: usize,           // from KernelContext process table
    pub vector_segments: usize,            // from VecStore segment count
}
```

### Consensus Stack Integration

```rust
pub struct ConsensusStack {
    pub pbft: PbftLayer,
    pub raft: RaftLayer,
    pub gossip: GossipLayer,
}

impl ConsensusStack {
    /// Route a syscall to the appropriate consensus layer.
    pub async fn submit(&self, syscall: &Syscall) -> ConsensusResult {
        match syscall {
            Syscall::StateMutate { .. } => self.pbft.propose(syscall).await,
            _ => Ok(()),  // non-mutating syscalls skip consensus
        }
    }
}
```

Metadata changes are submitted via `self.raft.propose()` from `SwarmOrchestrator` when configuration updates arrive through the MCP `tools/call` interface.

## Consequences

### Positive
- Byzantine tolerance for the audit log without imposing PBFT overhead on all 25 nodes
- Raft provides linearizable reads for configuration, preventing split-brain placement decisions
- Gossip scales to 25+ nodes with minimal bandwidth (3 messages per node per round)
- RPi4 nodes only run gossip, keeping their 4 GB RAM free for `VecSearch` workloads

### Negative
- Three consensus implementations increase code complexity in `rlmx-swarm`
- Cross-layer coordination needed: Raft leader must be a PBFT replica (Zone A constraint)
- Gossip convergence delay (up to 5s) means placement decisions may use stale load data

### Risks
- PBFT view changes during Zone A network partitions could stall `StateMutate` for 5-10s
- Raft cluster of 15 nodes is larger than typical deployments; log replication to 8 RPi5 nodes may saturate their 1 Gbps NICs under heavy metadata churn
- `ruvector-raft` crate is a dependency not yet proven at this scale

## Alternatives Considered

1. **Single Raft everywhere**: All 25 nodes in one Raft cluster. Rejected because Raft provides only crash fault tolerance; a compromised edge node could corrupt the audit log by replaying stale entries as leader.

2. **etcd-based consensus**: Run etcd on Zone A nodes, use etcd watches for configuration. Rejected because it introduces a Go dependency, requires separate process management, and does not integrate with the Rust-native `CapabilityToken` validation pipeline.

3. **Pure gossip with CRDTs**: Use conflict-free replicated data types for all state. Rejected because the witness-chained audit log requires total ordering that CRDTs cannot provide. Audit entries have causal dependencies (each entry's SHA-256 digest includes the previous entry's hash).

4. **HotStuff BFT**: More efficient than PBFT (O(n) messages vs O(n^2)). Considered but deferred to Phase 2; PBFT is simpler to implement and the 5-node Zone A cluster makes the quadratic overhead manageable.

## References
- `crates/rlmx-kernel/src/syscall.rs` -- `Syscall::StateMutate` variant
- `crates/rlmx-kernel/src/types.rs` -- `SyscallResult::StateMutated { witness_id, success }`
- `crates/rlmx-kernel/src/capability.rs` -- `CapabilityToken` revocation requires consistent propagation
- `crates/rlmx-rvf/` -- sealed container format used for Raft log persistence
- ADR-001 -- zone definitions and node hardware profiles
