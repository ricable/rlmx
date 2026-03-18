# ADR-009: Pull-Based Browser WASM Compute Pool

## Status
Proposed

## Date
2026-03-18

## Context
The RLMX frontend already ships `@ruvector/ruvllm-wasm` v2.0.2 in
`frontend/ruvllm-wasm/`, providing WASM SIMD matrix multiplication and
embedding computation via Web Workers and SharedArrayBuffer. Currently this
capability is used only for local UI features (chat template formatting,
parallel matmul benchmark). Meanwhile, the 25-node swarm has compute-intensive
tasks -- embedding generation for `VecInsert`, matrix operations for TRM
neural network inference, and experiment evaluation -- that could be offloaded
to idle browser tabs.

Browsers represent "free" compute that scales with the number of connected
users. A pull-based model (browsers request work) avoids NAT traversal
issues and works through corporate firewalls.

## Decision
Implement a pull-based browser WASM compute pool where browser tabs act as
stateless compute workers in the "Browser" zone of the swarm topology.

### Server Side

New file: `crates/rlmx-swarm/src/browser_pool.rs`

```rust
pub struct BrowserPool {
    task_queue: Arc<Mutex<VecDeque<ComputeTask>>>,
    in_flight: Arc<Mutex<HashMap<TaskId, InFlightEntry>>>,
    results: Arc<Mutex<HashMap<TaskId, ComputeResult>>>,
    timeout: Duration,  // default 30s
}

#[derive(Serialize, Deserialize)]
pub struct ComputeTask {
    pub id: TaskId,
    pub task_type: ComputeTaskType,
    pub payload: Vec<u8>,     // msgpack-encoded input
    pub created_at: Instant,
    pub priority: u8,
}

#[derive(Serialize, Deserialize)]
pub enum ComputeTaskType {
    Embedding { text: String, dim: usize },
    MatMul { a_shape: [usize; 2], b_shape: [usize; 2] },
    CosineSimilarity { query: Vec<f32>, candidates: Vec<Vec<f32>> },
    TrmForward { stream: TrmStream, input: Vec<f32> },
}
```

The `BrowserPool` integrates with the WebSocket server (ADR-008) on port 3001.
Browser workers connect to `ws://host:3001/compute` and enter a pull loop:

1. Worker sends `{ "action": "poll" }`.
2. Server responds with a `ComputeTask` or `{ "action": "wait", "retry_ms": 500 }`.
3. Worker computes the result using ruvllm-wasm APIs.
4. Worker sends `{ "action": "result", "task_id": "...", "payload": "..." }`.
5. Repeat.

### Browser Side

New file: `frontend/compute-worker.js`

```javascript
// Loaded as a Web Worker from the dashboard
import init, { ParallelInference, EmbeddingEngine } from './ruvllm-wasm/ruvllm_wasm.js';

let ws;
let inference;

async function connect(serverUrl) {
    await init();  // WASM init -- must call default() first, never init() after
    inference = new ParallelInference();

    ws = new WebSocket(`${serverUrl}/compute`);
    ws.onopen = () => ws.send(JSON.stringify({ action: 'poll' }));
    ws.onmessage = async (event) => {
        const msg = JSON.parse(event.data);
        if (msg.action === 'wait') {
            setTimeout(() => ws.send(JSON.stringify({ action: 'poll' })), msg.retry_ms);
            return;
        }
        const result = await executeTask(msg);
        ws.send(JSON.stringify({ action: 'result', task_id: msg.id, payload: result }));
        ws.send(JSON.stringify({ action: 'poll' }));  // immediately request next
    };
}
```

### Fault Tolerance

- **Tab close / disconnect**: When a WebSocket connection drops, all tasks
  in `in_flight` for that worker are returned to `task_queue` after a grace
  period (5 seconds). The `InFlightEntry` tracks `worker_id` and `assigned_at`.
- **Timeout**: Tasks not completed within `timeout` (30s default) are
  re-queued automatically by a background reaper task.
- **Duplicate results**: If a re-queued task completes from both the original
  and replacement worker, the first result wins; duplicates are discarded via
  `in_flight.remove()` returning `None`.

### Zone Classification

Browser workers belong to Zone::Browser in the swarm topology:
- **Stateless**: No persistent state, no SONA patterns, no RVF containers.
- **No consensus participation**: Browsers do not vote in Raft/hive-mind
  consensus (ADR-006). They are pure compute.
- **Capability-restricted**: Browser workers receive a minimal `CapabilityToken`
  with only `VecSearch` (read) permission. They cannot mutate state.

### Task Prioritization

The `task_queue` is a priority deque. Priority levels:
- `0` (highest): Embedding computation for active `VecInsert` syscalls.
- `1`: TRM forward pass for real-time inference.
- `2`: MatMul for experiment evaluation (ADR-006).
- `3` (lowest): Background similarity computation.

### Capacity Reporting

Browser workers report capabilities on connection:
```json
{
    "action": "register",
    "capabilities": {
        "wasm": true,
        "simd": true,
        "webgpu": false,
        "shared_array_buffer": true,
        "max_workers": 4
    }
}
```

The server uses this to route appropriate task types. WebGPU-capable browsers
receive larger MatMul tasks. SIMD-only browsers receive embedding tasks.

## Consequences

### Positive
- Turns every connected browser into a compute node at zero infrastructure cost.
- Pull-based model works through firewalls and NAT without WebRTC complexity.
- Leverages existing `@ruvector/ruvllm-wasm` v2.0.2 -- no new WASM compilation
  needed.
- Automatic work re-queuing on disconnect provides resilience against the
  inherently unreliable browser environment.
- Stateless design means no data loss when browsers close.

### Negative
- Browser compute is best-effort and unpredictable -- cannot be relied upon for
  latency-critical paths.
- SharedArrayBuffer requires `Cross-Origin-Isolation` headers (`COOP` + `COEP`),
  which may conflict with third-party integrations on the dashboard page.
- WASM compute is slower than native Rust -- approximately 2-4x for SIMD
  operations, more for non-SIMD fallback.

### Risks
- Malicious browser workers could return incorrect results. Mitigated by:
  (a) result validation for embedding tasks (check dimensionality and norm),
  (b) optional redundant computation (send same task to 2 workers, compare).
- High browser worker count could overwhelm the WebSocket server. Mitigated
  by connection limit in ADR-008 (64 max WS connections shared between event
  subscribers and compute workers).
- Privacy concern: task payloads (text for embedding) are sent to browsers.
  Mitigated by only sending non-sensitive synthetic text or pre-tokenized
  integer sequences.

## Alternatives Considered

1. **WebRTC P2P mesh**: Browsers communicate directly, forming a true P2P
   compute mesh. Eliminates server as bottleneck. However, WebRTC connection
   establishment is complex (STUN/TURN), NAT traversal is unreliable, and
   coordinating work distribution in a P2P mesh adds significant complexity.
   Rejected for Phase 2; may revisit for Phase 4+.

2. **Server-only compute**: Ignore browser compute entirely, run all workloads
   on Mac/NUC/RPi5 nodes. Simpler architecture, but wastes available browser
   resources and does not scale with user count. Rejected.

3. **WebGPU-only approach**: Restrict to browsers with WebGPU for maximum
   throughput. Too exclusionary -- WebGPU adoption is still limited (Chrome
   113+, no Firefox/Safari stable). WASM SIMD has broader support. Rejected
   as the sole approach; WebGPU is used opportunistically via capability
   reporting.

## References
- `frontend/ruvllm-wasm/` -- existing WASM bundle.
- `frontend/index.html` -- current WASM init pattern (`await mod.default()`).
- `crates/rlmx-kernel/src/vector.rs` -- `VecInsert`/`VecSearch` syscalls.
- `crates/rlmx-trm/` -- TRM neural network forward pass.
- ADR-008 -- WebSocket server on port 3001.
