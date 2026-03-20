# ADR-003: Neural Model Routing via FastGRNN

## Status
Proposed

## Date
2026-03-18

## Context

The current `Scheduler::auto_select()` in `crates/rlmx-kernel/src/scheduler.rs` uses a hand-coded heuristic to route queries to strategies:

```rust
fn auto_select(&self, query: &str) -> Strategy {
    let len = query.len();
    let has_code = query.contains("```") || query.contains("fn ") || query.contains("def ");
    let is_question = query.trim_end().ends_with('?');

    if self.config.edge_available && is_question && len < 200 {
        return Strategy::Edge;
    }
    if has_code || len > 500 {
        Strategy::Trm("default".into())
    } else if is_question && len < 200 {
        Strategy::Rlm
    } else {
        Strategy::Hybrid { triage: "auto-triage".into(), threshold: 0.5 }
    }
}
```

This heuristic has several limitations:
- No consideration of node load or zone availability (post ADR-001)
- Binary thresholds (200 chars, 500 chars) with no learned boundaries
- Cannot adapt to changing workload patterns over time
- Does not account for query complexity beyond simple pattern matching
- Missing the new `Strategy::Swarm` variant needed for cross-zone scatter-gather

The `rlmx-trm` crate already implements a 2-layer neural network with adaptive halting. The `rlmx-cognitive` crate has `DagOptimizer` which tracks strategy success rates. Neither is currently wired into the routing decision.

## Decision

Replace the heuristic router with a **FastGRNN-based neural router** using the `ruvector-tiny-dancer-core` crate, producing sub-millisecond routing decisions with online learning.

### New Module: `crates/rlmx-kernel/src/router.rs`

```rust
pub struct TinyDancerRouter {
    model: FastGrnnModel,          // from ruvector-tiny-dancer-core
    feature_extractor: FeatureExtractor,
    online_buffer: Vec<RoutingExample>,
    retrain_threshold: usize,      // retrain after 100 examples
}

pub struct RouterInput {
    pub query_length: f32,         // normalized to [0, 1] over [0, 2048]
    pub has_code: f32,             // 0.0 or 1.0
    pub is_question: f32,          // 0.0 or 1.0
    pub trigram_entropy: f32,      // character trigram entropy, normalized
    pub edge_available: f32,       // 0.0 or 1.0
    pub node_load: f32,            // from GossipLayer MetricsUpdate, [0, 1]
    pub zone_a_available: f32,     // fraction of Zone A nodes healthy
    pub zone_b_available: f32,     // fraction of Zone B nodes healthy
    pub token_count_estimate: f32, // estimated tokens, normalized
    pub recent_strategy_success: [f32; 5], // rolling success rate per strategy
}
// Total input dimension: 14

pub struct RouterOutput {
    pub scores: [f32; 5],          // softmax over [Rlm, Trm, Edge, Hybrid, Swarm]
    pub confidence: f32,           // max(scores)
    pub latency_ns: u64,           // inference time
}
```

### FastGRNN Model Architecture

- **Input**: 14-dimensional feature vector
- **Hidden**: FastGRNN cell with 32 hidden units, zeta=0.9, nu=0.1 (sparse gating)
- **Output**: Linear projection to 5 classes + softmax
- **Parameters**: ~1,600 weights (fits in L1 cache)
- **Latency**: <1ms on RPi4, <100us on Mac Studio

FastGRNN (Fast, Accurate, Stable, and Tiny GRU) is chosen over standard GRU because its sparse gating matrices reduce multiply-accumulate operations by 10x while maintaining accuracy, critical for the RPi4 nodes in Zone C that may need local routing decisions.

### New Strategy Variant

```rust
// crates/rlmx-kernel/src/scheduler.rs
pub enum Strategy {
    Rlm,
    Trm(String),
    Auto,
    Hybrid { triage: String, threshold: f32 },
    Edge,
    Swarm {                        // NEW
        scatter_zones: Vec<Zone>,
        gather_strategy: GatherStrategy,  // First, Majority, Consensus
        timeout: Duration,
    },
}
```

`Strategy::Swarm` enables cross-zone parallel execution. The router selects it when query complexity is high (trigram entropy > 0.7), multiple zones are healthy, and no single strategy has >0.8 confidence.

### Online Learning

The router learns from execution outcomes:

```rust
pub struct RoutingExample {
    pub input: RouterInput,
    pub chosen_strategy: usize,    // index into [Rlm, Trm, Edge, Hybrid, Swarm]
    pub reward: f32,               // from DagOptimizer success tracking
}
```

After every `retrain_threshold` (default: 100) completed queries, `TinyDancerRouter::retrain()` runs a single epoch of SGD over the buffer. The `rlmx-cognitive` crate's `DagOptimizer` provides the reward signal: latency-weighted success rate normalized to [0, 1].

Training runs on the local node (no cross-node gradient sharing). Each node's router adapts to its local workload distribution. Zone A nodes see more complex queries; Zone B nodes see more edge-eligible queries.

### Integration with Scheduler

```rust
impl Scheduler {
    pub fn resolve_strategy(
        &self,
        query: &str,
        hint: Option<&Strategy>,
        router: Option<&TinyDancerRouter>,
        node_ctx: Option<&SwarmNode>,
    ) -> Strategy {
        match hint {
            Some(Strategy::Auto) | None if router.is_some() => {
                let input = router.unwrap().extract_features(query, node_ctx);
                let output = router.unwrap().infer(&input);
                if output.confidence > 0.6 {
                    output.to_strategy()
                } else {
                    self.auto_select(query)  // fallback to heuristic
                }
            }
            Some(other) => other.clone(),
            None => self.auto_select(query),
        }
    }
}
```

The heuristic `auto_select()` is retained as a fallback when the neural router's confidence is below 0.6 or when no router is configured (single-node deployments without `rlmx-swarm`).

## Consequences

### Positive
- Sub-millisecond routing decisions that adapt to workload patterns
- Node-local adaptation means each zone develops routing behavior suited to its traffic
- Confidence gating with heuristic fallback ensures graceful degradation
- Tiny model (1,600 params) adds negligible memory overhead to any node

### Negative
- Online learning introduces non-determinism: same query may route differently over time
- Cold start: until 100 examples accumulate, the router falls back to heuristics
- `ruvector-tiny-dancer-core` adds a new dependency to `rlmx-kernel`

### Risks
- Reward signal from `DagOptimizer` may be noisy if inference backends have variable latency
- Router could develop a bias toward one strategy if early examples are skewed
- Per-node learning means routing behavior is not consistent across the cluster (by design, but may complicate debugging)

## Alternatives Considered

1. **Heuristic rules (current)**: Keep the `auto_select()` function with more conditions. Rejected because the number of features (14) and strategies (5) creates a combinatorial explosion of if/else branches that is hard to maintain and cannot adapt.

2. **LLM-based routing**: Use a small LLM (e.g., via `Strategy::Edge` with a routing-specific prompt) to classify queries. Rejected because even a 0.5B model takes 50-200ms per inference -- two orders of magnitude slower than FastGRNN and wasteful for a classification task.

3. **Static mapping from MCP roles**: Map `RbacRole` (Viewer/Operator/Engineer/Admin) to strategies. Rejected because role does not correlate with query complexity. An Admin may issue simple queries; a Viewer may ask complex graph questions.

4. **Multi-armed bandit (epsilon-greedy)**: Simpler than neural routing. Rejected because it cannot condition on query features -- it would explore strategies uniformly regardless of query type.

## References
- `crates/rlmx-kernel/src/scheduler.rs` -- `Scheduler::auto_select()`, `Strategy` enum
- `crates/rlmx-trm/` -- existing neural network (2-layer, 3-stream) for TRM scheduling
- `crates/rlmx-cognitive/` -- `DagOptimizer` for strategy success rate tracking
- `crates/rlmx-mcp/src/server.rs` -- RBAC roles (Viewer, Operator, Engineer, Admin, Auditor, System)
- ADR-001 -- zone definitions, `SwarmNode`, `SwarmOrchestrator`
- ADR-002 -- `GossipLayer::MetricsUpdate` provides `node_load` feature
