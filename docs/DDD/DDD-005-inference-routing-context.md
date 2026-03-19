# DDD-005: Inference Routing Bounded Context

## Overview

The Inference Routing context decides **how** and **where** each query is
processed. It spans the existing `Scheduler` in `rlmx-kernel`, the edge
inference engine in `rlmx-ruvllm`, the RLM recursive agent in `rlmx-rlm`,
and the TRM neural network in `rlmx-trm`. The planned enhancement adds a
sub-millisecond neural router (TinyDancer/FastGRNN) and tiered model
escalation.

**Crates**: `rlmx-kernel` (scheduler.rs), `rlmx-ruvllm`, `rlmx-rlm`, `rlmx-trm`

## Aggregate Root: RoutingEngine

```rust
pub struct RoutingEngine {
    pub scheduler: Scheduler,                // existing: crates/rlmx-kernel/src/scheduler.rs
    pub tiny_dancer: Option<TinyDancerRouter>, // planned: FastGRNN neural router
    pub tiered_engine: TieredEngine,         // planned: multi-tier model dispatch
    pub local_engine: Option<LocalEngine>,   // existing: crates/rlmx-ruvllm/src/engine.rs
    pub routing_history: Vec<RoutingRecord>,
}
```

The `RoutingEngine` is the aggregate root because it coordinates the routing
decision across all tiers and backends. It owns the invariant that routing
latency stays below 1ms for the fast path.

## Entities

### Scheduler (existing)

Defined in `crates/rlmx-kernel/src/scheduler.rs`:

```rust
pub struct Scheduler {
    pub config: SchedulerConfig,
}

pub struct SchedulerConfig {
    pub default_strategy: Strategy,
    pub max_recursion_depth: usize,    // default: 10
    pub query_timeout: Duration,       // default: 30s
    pub max_concurrent_processes: usize, // default: 64
    pub edge_available: bool,
}
```

Current `auto_select()` heuristic:
- Edge available + short question (<200 chars) -> `Strategy::Edge`
- Has code or long (>500 chars) -> `Strategy::Trm`
- Short question -> `Strategy::Rlm`
- Otherwise -> `Strategy::Hybrid`

### TinyDancerRouter (planned)

A FastGRNN (Fast, Gated Recurrent Neural Network) that replaces the heuristic
`auto_select()` with a learned routing function:

```rust
pub struct TinyDancerRouter {
    pub weights: FastGrnnWeights,
    pub input_dim: usize,   // 6 features
    pub hidden_dim: usize,  // 32
    pub output_dim: usize,  // 5 strategies
}
```

### LocalEngine (existing)

Defined in `crates/rlmx-ruvllm/src/engine.rs`:

- Wraps `ruvllm::CandleBackend` behind `#[cfg(feature = "ruvllm")]`
- Without the feature: stub returning `RuvllmError::NotAvailable`
- Methods: `load()`, `generate(prompt, max_tokens)`, `chat(messages, max_tokens)`
- Returns `GenerateResult { text, tokens_generated, latency_ms }`

### TieredEngine (planned)

Manages multiple model backends at different capability tiers:

```rust
pub struct TieredEngine {
    pub tiers: Vec<TierConfig>,
    pub active_backends: HashMap<ModelTier, Box<dyn InferenceBackend>>,
}

pub struct TierConfig {
    pub tier: ModelTier,
    pub backend_type: InferenceBackendType,
    pub model_spec: ModelSpec,
    pub max_latency_ms: u64,
    pub cost_per_token: f64,
}
```

### ModelSpec (existing)

Defined in `crates/rlmx-ruvllm/src/config.rs`:

```rust
pub struct ModelSpec {
    pub name: String,
    pub path: PathBuf,
    pub quantization: String,    // e.g., "Q4_K_M"
    pub context_length: usize,
}
```

## Value Objects

| Value Object | Location | Definition |
|-------------|----------|------------|
| `Strategy` | `scheduler.rs` | Enum: `Rlm`, `Trm(String)`, `Auto`, `Hybrid { triage, threshold }`, `Edge` |
| `ModelTier` | planned | Enum: `Small`, `Medium`, `Large`, `Edge`, `Custom(String)` |
| `HardwareBackend` | `config.rs` | Enum: `Cpu`, `Metal`, `Cuda`, `WebGpu`, `Auto` |
| `InferenceBackendType` | planned | Enum: `Candle`, `Mlx`, `VllmRemote`, `CloudApi` |
| `RouterInput` | planned | `{ query_length, has_code, is_question, trigram_entropy, edge_available, node_load }` |
| `RouterOutput` | planned | `{ scores: [f32; 5], selected: Strategy, confidence: f32, latency_us: u64 }` |
| `ConfidenceScore` | planned | Newtype `f64` in range [0.0, 1.0] |
| `EdgeConfig` | `config.rs` | `{ model, backend, max_memory_mb, max_tokens, temperature, threads }` |

## Domain Events

| Event | Trigger | Data |
|-------|---------|------|
| `QueryRouted` | `route()` completes | `{ query_hash, strategy, confidence, latency_us, node_id }` |
| `ModelEscalated` | Confidence below threshold | `{ query_hash, from_tier, to_tier, reason }` |
| `RoutingLearned` | `train_from_history()` updates weights | `{ samples_used, loss_delta, accuracy }` |
| `InferenceCompleted` | Backend returns result | `{ strategy, model_tier, tokens, latency_ms, quality }` |
| `EdgeModelLoaded` | `LocalEngine::load()` succeeds | `{ model_name, backend, memory_mb }` |
| `EdgeModelUnavailable` | Load fails or feature disabled | `{ reason }` |

## Domain Services

### `route(query, context) -> RouterOutput` (<1ms)

The fast path:

1. Extract features into `RouterInput`: query length, code detection, question
   detection, trigram entropy, edge availability, current node load.
2. If `TinyDancerRouter` is available: run FastGRNN forward pass (<1ms).
   Produces softmax over 5 strategies. Select highest-confidence strategy.
3. If not available: fall back to `Scheduler::auto_select()` heuristic.
4. Record `QueryRouted` event.

### `escalate(query, current_tier) -> Strategy`

When inference on the current tier produces confidence below threshold:

1. Look up next tier: `Edge -> Small -> Medium -> Large -> Cloud`
2. Check if next tier is available on this node or requires swarm routing.
3. If swarm routing needed, emit `ModelEscalated` and delegate to Swarm
   Coordination context for node selection.
4. Return the escalated `Strategy`.

### `train_from_history(records: &[RoutingRecord])`

Online learning for TinyDancer:

1. Collect `(RouterInput, actual_best_strategy)` pairs from completed queries.
2. Run mini-batch gradient descent on FastGRNN weights.
3. Emit `RoutingLearned` event with loss metrics.
4. Integrates with SONA's `PatternBank` for long-term pattern storage.

## Invariants

1. **Routing latency <1ms**: The fast path (feature extraction + FastGRNN
   forward pass) must complete within 1 millisecond. No network calls, no
   mutex contention on the hot path.

2. **Escalation monotonicity**: Escalation only moves to a higher tier, never
   to a lower one within a single query lifecycle. `Edge < Small < Medium < Large < Cloud`.

3. **Stub safety**: When compiled without `ruvllm` feature, `LocalEngine`
   returns `NotAvailable` -- never panics. Edge MCP tools return
   `"status": "unavailable"`.

4. **Edge model requirements**: A GGUF model requires a companion
   `<model-name>-tokenizer.json` file in the same directory. `load()` fails
   gracefully if the tokenizer is missing.

5. **Recursion depth**: The `Scheduler` enforces `max_recursion_depth` (default
   10) for RLM recursive agent calls. Checked via `check_recursion_depth()`.

6. **Concurrent process cap**: `SchedulerConfig::max_concurrent_processes`
   (default 64) limits total in-flight inference processes.

## Routing Decision Flow

```
Query arrives
    |
    v
+-------------------+
| Extract features  |  query_length, has_code, is_question,
| (RouterInput)     |  trigram_entropy, edge_available, node_load
+-------------------+
    |
    v
+-------------------+     No      +-------------------+
| TinyDancer avail? |------------>| Scheduler heuristic|
| (FastGRNN)        |             | auto_select()      |
+-------------------+             +-------------------+
    | Yes                              |
    v                                  v
+-------------------+          +-------------------+
| FastGRNN forward  |          | Rule-based select |
| softmax -> 5 strs |          |                   |
| <1ms              |          |                   |
+-------------------+          +-------------------+
    |                                  |
    +----------------------------------+
    |
    v
+-------------------+
| Strategy selected |
+-------------------+
    |
    +---> Rlm:   VllmClient -> cloud LLM (constrained 5-action grammar)
    +---> Trm:   2-layer NN, 3 streams, adaptive halting
    +---> Edge:  LocalEngine -> CandleBackend (Candle/Metal/CUDA)
    +---> Hybrid: Triage model classifies, then dispatches
    +---> Swarm:  Route to best node (planned)
```

## File Map

| File | Types | Status |
|------|-------|--------|
| `crates/rlmx-kernel/src/scheduler.rs` | `Strategy`, `Scheduler`, `SchedulerConfig` | Existing |
| `crates/rlmx-ruvllm/src/engine.rs` | `LocalEngine`, `GenerateResult`, `ChatMessage` | Existing |
| `crates/rlmx-ruvllm/src/config.rs` | `EdgeConfig`, `HardwareBackend`, `ModelSpec` | Existing |
| `crates/rlmx-ruvllm/src/model.rs` | `ModelManager` (discovers GGUF files) | Existing |
| `crates/rlmx-ruvllm/src/error.rs` | `RuvllmError` | Existing |
| `crates/rlmx-rlm/src/` | `VllmClient`, recursive agent grammar | Existing |
| `crates/rlmx-trm/src/` | 2-layer NN, 3 streams, halting | Existing |
| `crates/rlmx-kernel/src/router.rs` | `TinyDancerRouter`, `RouterInput`, `RouterOutput` | Planned |
| `crates/rlmx-ruvllm/src/tiered.rs` | `TieredEngine`, `TierConfig` | Planned |
