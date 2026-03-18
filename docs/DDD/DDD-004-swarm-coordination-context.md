# DDD-004: Swarm Coordination Bounded Context

## Overview

The Swarm Coordination context manages the distributed runtime: a 25-node
cluster organized into 5 zones, with pluggable consensus (PBFT, Raft, Gossip),
fault detection, and zone failover. It is the **infrastructure backbone** that
the Agent Lifecycle and Inference Routing contexts depend on.

**Crate**: `crates/rlmx-swarm/` (planned)

## Aggregate Root: SwarmCluster

```rust
pub struct SwarmCluster {
    pub id: Uuid,
    pub config: SwarmConfig,
    pub nodes: HashMap<NodeId, SwarmNode>,
    pub zones: HashMap<ZoneId, Zone>,
    pub consensus: Box<dyn ConsensusLayer>,
    pub transport: SwarmTransport,
    pub leader: Option<NodeId>,
    pub epoch: u64,
}
```

`SwarmCluster` is the aggregate root because it owns the consistency boundary
for node membership, leader election, and zone assignments. All mutations to
cluster state flow through it.

## Entities

### SwarmNode

```rust
pub struct SwarmNode {
    pub id: NodeId,
    pub zone: ZoneId,
    pub profile: NodeProfile,
    pub agents: Vec<AgentId>,
    pub health: HealthStatus,
    pub joined_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub role: NodeRole, // Leader, Follower, Candidate
}
```

### Zone

5 zones with distinct hardware profiles and failure domains:

| Zone | Purpose | Typical Hardware | Max Nodes |
|------|---------|-----------------|-----------|
| `ZoneA` | Primary compute, low latency | M-series Mac / x86 server | 8 |
| `ZoneB` | Secondary compute, overflow | Same or weaker hardware | 8 |
| `ZoneC` | Edge / embedded | RPi5, Jetson Nano | 5 |
| `Cloud` | Cloud escalation, heavy models | GPU instances (A100, H100) | 3 |
| `Browser` | In-browser WASM inference | WebGPU-capable browser tabs | 1 |

```rust
pub struct Zone {
    pub id: ZoneId,
    pub nodes: Vec<NodeId>,
    pub coordinator_agent: Option<AgentId>,
    pub failover_target: Option<ZoneId>,
    pub max_nodes: usize,
}
```

### ConsensusLayer

Three pluggable consensus implementations:

| Protocol | Use Case | Requirement | Latency |
|----------|----------|-------------|---------|
| **PBFT** | Critical state (leader election, token issuance) | 3f+1 nodes for f faults | ~100ms |
| **Raft** | Log replication, steady-state coordination | Majority quorum (n/2+1) | ~50ms |
| **Gossip** | Membership, health propagation, soft state | Eventual convergence <10s | ~1-5s |

```rust
pub trait ConsensusLayer: Send + Sync {
    fn propose(&mut self, proposal: ConsensusProposal) -> ConsensusResult;
    fn vote(&mut self, proposal_id: Uuid, vote: Vote) -> ConsensusResult;
    fn commit(&mut self, proposal_id: Uuid) -> ConsensusResult;
    fn leader(&self) -> Option<NodeId>;
    fn is_quorum_met(&self) -> bool;
}
```

### SwarmTransport

```rust
pub struct SwarmTransport {
    pub protocol: TransportProtocol, // Tcp, Quic, WebSocket, InProcess
    pub port: u16,
    pub tls_enabled: bool,
    pub max_message_size: usize,
}
```

## Value Objects

| Value Object | Definition |
|-------------|------------|
| `NodeId` | Newtype wrapping `Uuid`. Identifies a physical/virtual node. |
| `ZoneId` | Enum: `ZoneA`, `ZoneB`, `ZoneC`, `Cloud`, `Browser` |
| `NodeProfile` | `{ cpu_cores, memory_mb, gpu_available, backend: HardwareBackend, os, arch }` |
| `SwarmConfig` | `{ max_nodes: 25, heartbeat_interval, election_timeout, consensus_protocol, zones }` |
| `ConsensusConfig` | `{ protocol, quorum_size, timeout, max_retries }` |
| `HealthStatus` | `{ status: Healthy/Degraded/Unreachable, latency_ms, cpu_load, memory_pct, last_check }` |
| `NodeRole` | Enum: `Leader`, `Follower`, `Candidate` |
| `Vote` | Enum: `Accept`, `Reject(String)` |
| `ConsensusProposal` | `{ id, proposer, action: ProposalAction, timestamp }` |
| `ProposalAction` | Enum: `ElectLeader(NodeId)`, `AddNode(NodeId)`, `RemoveNode(NodeId)`, `MigrateAgent(AgentId, NodeId)` |

## Domain Events

| Event | Trigger | Data |
|-------|---------|------|
| `NodeJoined` | New node registers with cluster | `{ node_id, zone, profile }` |
| `NodeLeft` | Graceful departure or detected failure | `{ node_id, reason }` |
| `LeaderElected` | Consensus completes election | `{ leader_id, epoch, votes_received }` |
| `ConsensusReached` | Proposal committed | `{ proposal_id, action, participants }` |
| `PartitionDetected` | Heartbeat failures exceed threshold | `{ zone, unreachable_nodes, detected_at }` |
| `ZoneFailover` | Zone loses all healthy nodes | `{ failed_zone, target_zone, migrated_agents }` |
| `HeartbeatMissed` | Node fails to respond within interval | `{ node_id, consecutive_misses }` |
| `AgentMigrated` | Agent moved between nodes | `{ agent_id, from_node, to_node, reason }` |

## Repositories

| Repository | Purpose |
|-----------|---------|
| `NodeRegistry` | CRUD for `SwarmNode` instances. In-memory `HashMap<NodeId, SwarmNode>`. |
| `TopologyStore` | Persists the cluster topology graph (which nodes connect to which). |
| `EpochLog` | Append-only log of consensus epochs and leader transitions. |

## Domain Services

### SwarmOrchestrator

Manages the full cluster lifecycle:

```rust
impl SwarmOrchestrator {
    /// Bootstrap a new cluster with initial nodes.
    pub async fn bootstrap(config: SwarmConfig, seed_nodes: Vec<NodeProfile>) -> SwarmCluster;

    /// Add a node to the cluster (requires consensus).
    pub async fn add_node(cluster: &mut SwarmCluster, profile: NodeProfile) -> Result<NodeId>;

    /// Remove a node gracefully (migrates its agents first).
    pub async fn remove_node(cluster: &mut SwarmCluster, node_id: NodeId) -> Result<()>;

    /// Rebalance agents across nodes in a zone.
    pub async fn rebalance(cluster: &mut SwarmCluster, zone: ZoneId) -> Result<Vec<AgentMigrated>>;

    /// Trigger leader election for the given zone.
    pub async fn elect_leader(cluster: &mut SwarmCluster, zone: ZoneId) -> Result<NodeId>;
}
```

### HealthMonitor

```rust
impl HealthMonitor {
    /// Run periodic heartbeat checks against all nodes.
    pub async fn heartbeat_loop(cluster: &SwarmCluster, interval: Duration);

    /// Evaluate node health and predict failures using neuro-divergent
    /// forecasting (integrates with rlmx-cognitive's DagOptimizer for
    /// latency trend analysis).
    pub fn predict_failure(node: &SwarmNode, history: &[HealthStatus]) -> f64;

    /// Detect network partitions by analyzing heartbeat patterns.
    pub fn detect_partition(cluster: &SwarmCluster) -> Option<PartitionDetected>;
}
```

### FaultInjector (chaos testing)

```rust
impl FaultInjector {
    /// Simulate a node crash for testing failover.
    pub async fn crash_node(cluster: &mut SwarmCluster, node_id: NodeId);

    /// Simulate network partition between two zones.
    pub async fn partition_zones(cluster: &mut SwarmCluster, zone_a: ZoneId, zone_b: ZoneId);

    /// Simulate high latency on a node.
    pub async fn inject_latency(cluster: &mut SwarmCluster, node_id: NodeId, latency: Duration);
}
```

## Invariants

1. **PBFT fault tolerance**: For PBFT consensus, the cluster requires at
   least `3f + 1` nodes to tolerate `f` Byzantine faults. With 25 nodes,
   the system tolerates up to 8 Byzantine faults.

2. **Raft majority quorum**: Raft requires `n/2 + 1` nodes for any committed
   decision. With 25 nodes, quorum = 13.

3. **Gossip convergence**: Gossip protocol must converge membership state
   across all nodes within 10 seconds under normal conditions.

4. **Zone failover constraints**: When a zone fails, its agents are migrated
   to the designated `failover_target` zone. Migration respects the target
   zone's `max_nodes` limit. If the target is also full, the cluster enters
   degraded mode rather than violating capacity constraints.

5. **Single leader per epoch**: Each consensus epoch has at most one leader.
   The epoch counter is monotonically increasing.

6. **Heartbeat liveness**: A node is marked `Unreachable` after 3 consecutive
   missed heartbeats (configurable). A zone with >50% unreachable nodes
   triggers `PartitionDetected`.

7. **Node cap**: Maximum 25 nodes in the cluster. `add_node()` rejects if
   at capacity.

## Topology Diagram (25-node example)

```
Zone A (Primary)                Zone B (Secondary)
+--------+--------+            +--------+--------+
| Node 1 | Node 2 |            | Node 9 | Node 10|
| Leader  | Follow |            | Leader  | Follow |
+--------+--------+            +--------+--------+
| Node 3 | Node 4 |            | Node 11| Node 12|
+--------+--------+            +--------+--------+
| Node 5 | Node 6 |            | Node 13| Node 14|
+--------+--------+            +--------+--------+
| Node 7 | Node 8 |            | Node 15| Node 16|
+--------+--------+            +--------+--------+

Zone C (Edge)        Cloud           Browser
+---------+          +---------+     +---------+
| RPi5-01 |          | GPU-01  |     | WASM-01 |
| RPi5-02 |          | GPU-02  |     +---------+
| RPi5-03 |          | GPU-03  |
| RPi5-04 |          +---------+
| RPi5-05 |
+---------+

Consensus: PBFT across zone leaders, Raft within zones, Gossip for membership
Transport: QUIC between zones, TCP within zones, WebSocket to Browser
```

## File Map (planned)

| File | Types |
|------|-------|
| `crates/rlmx-swarm/src/node.rs` | `SwarmNode`, `NodeProfile`, `NodeRole`, `HealthStatus` |
| `crates/rlmx-swarm/src/cluster.rs` | `SwarmCluster`, `SwarmConfig` |
| `crates/rlmx-swarm/src/zone.rs` | `Zone`, `ZoneId` |
| `crates/rlmx-swarm/src/consensus.rs` | `ConsensusLayer` trait, `ConsensusConfig`, PBFT/Raft/Gossip impls |
| `crates/rlmx-swarm/src/transport.rs` | `SwarmTransport`, `TransportProtocol` |
| `crates/rlmx-swarm/src/orchestrator.rs` | `SwarmOrchestrator` |
| `crates/rlmx-swarm/src/health.rs` | `HealthMonitor`, failure prediction |
| `crates/rlmx-swarm/src/chaos.rs` | `FaultInjector` |
| `crates/rlmx-swarm/src/lib.rs` | Public re-exports |
