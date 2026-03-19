# Feature Gates (ADR-024)

Ruvnet ecosystem crates are integrated behind opt-in feature gates. Stub implementations remain the default — ruvnet crates activate only when features are enabled.

## Feature Matrix

| Feature | Crate | What It Enables |
|---------|-------|-----------------|
| `ruvector` | `rlmx-kernel` | Core ruvector dependencies (HNSW, TinyDancer, attention, GNN) |
| `ruvnet-phase1` | `rlmx-kernel` | Vector & Storage: BloomScreenedRegion, NamespacedMemoryStore, EnhancedGraph via ruvector-graph/filter/collections |
| `ruvnet-phase2` | `rlmx-swarm` | Consensus & Networking: QuDagConsensusLayer, RuVectorRaftConsensus, TransportManager, SwarmDiscovery via qudag/ruvector-raft/ruv-swarm |
| `ruvnet-phase3` | `rlmx-ruvllm`, `rlmx-cognitive` | Enhanced Inference: BudgetOptimizer, AttentionOptimizer via ruvector-attention/sona/solver; LifeGraphGnn, CognitiveFramework via cognitum |
| `ruvnet-phase4` | `rlmx-ruvllm` | Neural Integration: ruv-neural-core/signal/graph/embed/memory + ruvector-cognitive-container |
| `ruvnet-phase5` | `rlmx-kernel` | Bare-Metal Verification: MacaroonCapabilityManager, Phase5ProofEngine, IsolatedMemoryRegion via ruvix-cap/types/region |
| `rvf-ext` | `rlmx-rvf` | Extended RVF: rvf-types/runtime/wire/index/quant/crypto |
| `midstreamer` | `rlmx-kernel` | Midstreamer scheduler integration |
| `verified` | `rlmx-kernel` | Formal verification via ruvector-verified |
| `ruvllm` | `rlmx-ruvllm` | Edge inference via Candle backend |
| `metal` | `rlmx-ruvllm` | Apple Metal GPU acceleration |
| `napi` | `rlmx-napi` | NAPI-RS Node.js native bindings |
| `wasm` | `rlmx-wasm` | wasm-bindgen browser bindings |

## Rules

- Phase features are additive: phase N may include phase N-1
- External crate integration always has a stub fallback path for `#[cfg(not(feature = "..."))]`
- `cargo build --workspace` without features MUST compile clean (stub build invariant)
- Every phase-gated module needs both `#[cfg(feature = "X")]` and `#[cfg(not(feature = "X"))]` paths
