# ADR-008: WebSocket Server for Real-Time Swarm Event Streaming

## Status
Proposed

## Date
2026-03-18

## Context
The current RLMX MCP server (`crates/rlmx-mcp/src/http.rs`) exposes a single
`POST /mcp` endpoint for JSON-RPC 2.0 tool calls. This request-response model
works for command execution but cannot push real-time events to clients. The
25-node distributed swarm generates continuous state changes -- agents spawning,
nodes joining/leaving, experiments completing, mutations discovered -- that
dashboards and monitoring tools need to observe with sub-second latency.

The planned Svelte dashboard (Phase 5) requires two communication channels:
tool invocations via MCP (request-response) and event streaming (server-push).
Polling the MCP endpoint would waste bandwidth and introduce latency proportional
to the poll interval.

## Decision
Add a WebSocket server on port 3001 alongside the existing MCP HTTP server on
port 3000. Implementation in `crates/rlmx-mcp/src/ws.rs` using
`tokio-tungstenite` (already compatible with the Tokio runtime used throughout
RLMX).

### Event Types

```rust
// crates/rlmx-mcp/src/ws.rs
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum SwarmEvent {
    NodeJoined { node_id: Uuid, zone: Zone, capabilities: Vec<String> },
    NodeLeft { node_id: Uuid, reason: LeaveReason },
    AgentSpawned { agent_id: Uuid, agent_type: String, node_id: Uuid },
    AgentTerminated { agent_id: Uuid, reason: TerminationReason },
    HealthUpdate { node_id: Uuid, cpu: f32, mem_mb: u64, gpu_util: Option<f32> },
    ExperimentUpdate { experiment_id: Uuid, generation: u32, val_bpb: f64, status: ExpStatus },
    MutationFound { mutation_id: Uuid, fitness: f64, generation: u32, parent_id: Option<Uuid> },
}

#[derive(Serialize, Deserialize)]
pub enum Zone { Mac, Nuc, Edge, Browser }

#[derive(Serialize, Deserialize)]
pub enum LeaveReason { Graceful, Timeout, Crashed }

#[derive(Serialize, Deserialize)]
pub enum TerminationReason { Completed, Failed, Cancelled, ResourceLimit }

#[derive(Serialize, Deserialize)]
pub enum ExpStatus { Running, Completed, Failed, CrossPollinated }
```

### Subscription Protocol

Clients connect to `ws://host:3001/events` and send a JSON subscription message:

```json
{
    "action": "subscribe",
    "event_types": ["NodeJoined", "NodeLeft", "ExperimentUpdate"],
    "filters": {
        "node_id": "optional-uuid-filter",
        "zone": "Mac"
    }
}
```

The server maintains a per-connection filter set. Events not matching the
subscription are dropped server-side, reducing bandwidth. Clients can update
subscriptions at any time by sending another subscribe message. An unsubscribe
message removes specific event types:

```json
{ "action": "unsubscribe", "event_types": ["HealthUpdate"] }
```

### Architecture

```
                     +------------------+
                     |  SwarmEventBus   |
                     | (tokio broadcast)|
                     +--------+---------+
                              |
              +---------------+---------------+
              |               |               |
        +-----+-----+  +-----+-----+  +------+------+
        | WS Client |  | WS Client |  | Internal    |
        | (Dashboard)|  | (Monitor) |  | Subscriber  |
        +-----+-----+  +-----------+  | (Logger)    |
              |                        +-------------+
              v
        Filtered by
        subscription
```

`SwarmEventBus` is a `tokio::sync::broadcast::Sender<SwarmEvent>` with a
configurable buffer (default 1024 events). Internal components publish events
via `event_bus.send(event)`. The WebSocket handler task receives from a
`broadcast::Receiver`, applies the client's filter, and forwards matching
events as JSON text frames.

### Integration Points

- **Kernel**: `KernelContext` gains an `Option<SwarmEventBus>` field. Syscall
  handlers for `ProcessFork` and `StateMutate` publish `AgentSpawned` and
  `MutationFound` events.
- **Swarm layer**: Node join/leave and health monitoring publish `NodeJoined`,
  `NodeLeft`, `HealthUpdate`.
- **Research workflow** (ADR-006): Experimenters publish `ExperimentUpdate`;
  Coordinator publishes `MutationFound` on winner selection.
- **MCP server** (`server.rs`): `McpConfig` gains `ws_port: u16` (default 3001).
  `cmd_serve` in CLI spawns both HTTP and WS listeners.

### Connection Management

- Heartbeat: server sends WebSocket ping every 30 seconds. Clients that miss
  3 consecutive pongs are disconnected.
- Backpressure: if a client's outbound buffer exceeds 256 messages, the
  connection is closed with code 1008 (Policy Violation) to protect server
  memory.
- Authentication: the first message after connection must be a JSON object with
  an `auth_token` field. The token is resolved against the existing
  `token_roles` map in `McpConfig`. Unauthenticated connections are closed
  after 5 seconds. RBAC filtering applies: Viewer role sees all read events;
  `AgentSpawned`/`AgentTerminated` details require Operator+.

## Consequences

### Positive
- Sub-second event delivery to dashboards and monitoring tools.
- Server-side filtering reduces bandwidth -- clients only receive relevant events.
- `tokio::sync::broadcast` is zero-allocation for events that have no subscribers.
- Reuses existing RBAC model -- no new authentication mechanism needed.
- Clean separation: MCP for commands, WebSocket for events.

### Negative
- New dependency: `tokio-tungstenite` added to `rlmx-mcp/Cargo.toml`.
- Second port (3001) requires additional firewall/NAT configuration in
  production deployments.
- Broadcast channel drops events if a slow consumer falls behind the buffer.
  Mitigated by the 256-message backpressure disconnect.

### Risks
- WebSocket connections are long-lived and consume memory per client. A large
  number of dashboard instances could pressure the MCP server. Mitigated by
  connection limit (default 64 concurrent WS connections).
- Event ordering is best-effort across nodes. Two events from different nodes
  may arrive out of causal order. Acceptable for dashboard display; the
  witness chain in `rlmx-rvf` provides causal ordering when needed.

## Alternatives Considered

1. **Server-Sent Events (SSE)**: Simpler protocol, HTTP-native, but
   unidirectional -- clients cannot send subscription filters without a
   separate HTTP endpoint. Also lacks binary frame support for future
   compressed payloads. Rejected.

2. **Polling MCP endpoints**: Adding `rlmx_swarm_events` as a tool that
   returns buffered events since last call. Simple but introduces poll-interval
   latency (minimum 1s realistic), wastes bandwidth on empty polls, and
   requires server-side per-client event buffering. Rejected.

3. **gRPC streaming**: Strong typing via protobuf, built-in flow control.
   However, adds `tonic`/`prost` dependencies, requires protobuf toolchain,
   and is not natively supported by browsers without grpc-web proxy. Rejected
   for this phase; may reconsider for inter-node communication later.

## References
- `crates/rlmx-mcp/src/http.rs` -- existing HTTP transport.
- `crates/rlmx-mcp/src/server.rs` -- `McpConfig`, `token_roles`, RBAC model.
- `tokio-tungstenite` crate: https://docs.rs/tokio-tungstenite
- `tokio::sync::broadcast` documentation.
- ADR-006 -- evolutionary auto-research events.
