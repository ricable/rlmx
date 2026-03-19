# ADR-001: Hierarchical Zone-Based Swarm Topology

## Status
Proposed

## Date
2026-03-18

## Context

RLMX is expanding from a single-node cognition kernel (9 crates, Tokio async runtime) to a 25-node distributed swarm spanning heterogeneous hardware:

- **3x Mac Studio M2 Ultra** (192 GB unified, Metal GPU) -- high-end inference and coordination
- **4x Intel NUC 13 Pro** (2 with NVIDIA T4 GPUs, 2 CPU-only) -- medium inference, metadata services
- **8x Raspberry Pi 5** (8 GB, ARM Cortex-A76) -- edge inference with GGUF models via `rlmx-ruvllm`
- **6x Raspberry Pi 4** (4 GB, ARM Cortex-A72) -- telemetry, gossip, lightweight vector search
- **2x Cloud burst VMs** (on-demand GPU, NVIDIA A10G) -- overflow for large-model inference
- **2x Browser WASM pools** (WebGPU/SIMD via `@ruvector/ruvllm-wasm`) -- client-side matmul, chat template formatting

The current `rlmx-kernel` Scheduler (`crates/rlmx-kernel/src/scheduler.rs`) routes queries to `Strategy::Rlm`, `Strategy::Trm`, `Strategy::Edge`, `Strategy::Auto`, or `Strategy::Hybrid` on a single node. It has no concept of multi-node placement, network topology, or zone-aware scheduling. The `rlmx-mcp` server (`crates/rlmx-mcp/src/server.rs`) binds to a single HTTP/stdio transport with no clustering.

We need an architecture that:
1. Respects hardware heterogeneity (GPU nodes handle heavy inference, RPi4 nodes do not)
2. Minimizes cross-zone latency for latency-sensitive operations
3. Allows graceful degradation when zones go offline
4. Integrates with the existing `CapabilityToken` security model (`crates/rlmx-kernel/src/capability.rs`)

## Decision

Adopt a **hierarchical zone-based swarm topology** with four zones and zone-aware placement policy.

### Zone Definitions

| Zone | Nodes | Role | Latency Budget |
|------|-------|------|----------------|
| **A (Compute)** | 3 Mac Studio + 2 NUC-GPU | Coordination, PBFT leader, heavy inference (RLM, large TRM) | <50ms intra-zone |
| **B (Inference)** | 8 RPi5 + 2 NUC-CPU | Edge inference (`Strategy::Edge`), Raft voters, vector search | <100ms intra-zone |
| **C (Edge)** | 6 RPi4 | Gossip, telemetry aggregation, lightweight `VecSearch` | <200ms intra-zone |
| **D (Burst)** | 2 Cloud VMs + 2 Browser pools | Overflow inference, client-side WASM computation | Variable |

### New Crate: `rlmx-swarm`

A new workspace member `crates/rlmx-swarm` will be added to `Cargo.toml` with these core types:

```rust
// crates/rlmx-swarm/src/node.rs
pub struct SwarmNode {
    pub id: NodeId,
    pub zone: Zone,
    pub capabilities: NodeCapabilities,  // GPU, memory, supported strategies
    pub endpoint: SocketAddr,
    pub health: Arc<RwLock<HealthState>>,
}

// crates/rlmx-swarm/src/cluster.rs
pub struct SwarmCluster {
    pub nodes: HashMap<NodeId, SwarmNode>,
    pub zones: HashMap<Zone, Vec<NodeId>>,
    pub placement_policy: PlacementPolicy,
    pub topology: TopologyGraph,  // uses rlmx-kernel's graph subsystem
}

// crates/rlmx-swarm/src/orchestrator.rs
pub struct SwarmOrchestrator {
    pub cluster: Arc<RwLock<SwarmCluster>>,
    pub scheduler: Scheduler,       // from rlmx-kernel
    pub consensus: ConsensusStack,  // from ADR-002
    pub router: TinyDancerRouter,   // from ADR-003
}
```

### Placement Policy

`PlacementPolicy` maps `Strategy` variants to eligible zones:

| Strategy | Primary Zone | Fallback | Rationale |
|----------|-------------|----------|-----------|
| `Rlm` | A | D (cloud burst) | Requires GPU for vLLM |
| `Trm` | A | B (NUC-CPU for small models) | Neural network forward pass |
| `Edge` | B | C (degraded, RPi4) | GGUF inference via `LocalEngine` |
| `Auto` | Resolved by `TinyDancerRouter` | -- | See ADR-003 |
| `Hybrid` | A (triage) then zone per sub-strategy | -- | Triage on GPU, dispatch to resolved zone |
| `Swarm` (new) | Cross-zone scatter-gather | -- | Parallel execution across zones |

### Simulation Layer

`rlmx-swarm` wraps `ruvix-qemu-swarm` for local development, simulating 25 nodes as Tokio tasks with configurable synthetic latency per zone pair. Production runtime replaces simulation with real gRPC transport (`tonic`).

### Integration Points

- `Scheduler::resolve_strategy()` gains a `node_context: Option<&SwarmNode>` parameter
- `KernelContext::dispatch()` calls are forwarded to remote nodes via `SwarmOrchestrator::route_syscall()`
- `CapabilityToken` tokens include a `zone_scope: Option<Zone>` field restricting token validity to specific zones
- MCP server (`rlmx-mcp`) adds `swarm_status` and `swarm_route` tools to the existing 15-tool set

## Consequences

### Positive
- Hardware-appropriate scheduling: GPU-heavy work stays on Mac/NUC-GPU, lightweight work on RPi
- Zone isolation limits blast radius of node failures to a single zone
- Simulation layer enables full-swarm testing on a single developer machine
- Incremental adoption: existing single-node deployments continue to work (zone A only)

### Negative
- New crate adds to workspace build time (~15s incremental on Mac Studio)
- Cross-zone syscall forwarding adds 2-10ms latency per hop
- Placement policy requires manual tuning for new hardware profiles

### Risks
- Zone D (cloud burst) introduces external dependency and variable latency
- Browser WASM pool nodes have unreliable availability (tab closure, sleep)
- 25-node cluster may exceed Tokio's default thread pool on RPi4 (4 cores)

## Alternatives Considered

1. **Flat mesh topology**: Every node peers with every other node. Rejected because RPi4 nodes cannot maintain 24 peer connections; O(n^2) gossip overhead is prohibitive at 25 nodes.

2. **Single-leader star topology**: One Mac Studio coordinates all traffic. Rejected because it creates a single point of failure and a bandwidth bottleneck (all inference results route through one node).

3. **Pure cloud deployment**: Run all 25 nodes as cloud VMs. Rejected because it eliminates the edge inference use case (RPi5 deployment for `deploy/rlmx-edge.service`), increases cost, and adds latency for local queries.

4. **Kubernetes-based orchestration**: Use k3s on all nodes. Rejected because RPi4 nodes lack memory for kubelet overhead (~500 MB), and the kernel's capability model already provides process isolation.

## References
- `crates/rlmx-kernel/src/scheduler.rs` -- existing `Strategy` enum and `Scheduler`
- `crates/rlmx-kernel/src/capability.rs` -- `CapabilityToken` with HMAC-SHA256 signatures
- `crates/rlmx-kernel/src/syscall.rs` -- 12-variant `Syscall` enum
- `crates/rlmx-mcp/src/server.rs` -- MCP server with RBAC and initialization handshake
- `deploy/rlmx-edge.service` -- systemd unit for RPi5 edge deployment
- `frontend/ruvllm-wasm/` -- `@ruvector/ruvllm-wasm` v2.0.2 browser inference
