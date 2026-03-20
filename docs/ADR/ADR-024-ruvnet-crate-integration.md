# ADR-024: Ruvnet Crate Integration Strategy

Status: Implemented

## Context
The RuVix Mesh PRD references 40+ external ruvnet crates that replace current stub implementations in the RLMX kernel. These crates span vector storage, consensus, inference, neural primitives, and bare-metal HAL. Integration must be phased, feature-gated, and must not break the stub build (`cargo build --workspace` without features MUST compile clean).

## Decision
Integrate ruvnet crates in 5 phases behind feature gates, following the existing pattern of opt-in features (like `ruvllm` and `metal`). Each phase has a dedicated feature flag. Stub implementations remain the default — ruvnet crates activate only when features are enabled.

### Phase 1: Vector & Storage (`ruvnet-phase1` feature)
| Ruvnet Crate | RLMX Target | Replaces |
|---|---|---|
| `ruvector-core` | `rlmx-kernel/src/memory.rs` | Brute-force MemoryRegion → HNSW-indexed vector storage |
| `ruvector-graph` | `rlmx-kernel/src/memory.rs` | Stub Graph → production graph store |
| `ruvector-filter` | `rlmx-kernel/src/memory.rs` | No bloom filter → bloom pre-screening on VecSearch |
| `ruvector-collections` | `rlmx-kernel/src/memory.rs` | Namespace isolation per agent domain at vector level |
| `rvf-types` | `rlmx-rvf` | Wire format types (already partially integrated) |
| `rvf-runtime` | `rlmx-rvf` | Agent package loading and execution |
| `rvf-wire` | `rlmx-rvf` | Device-to-device sync serialization |
| `rvf-index` | `rlmx-rvf` | Fast agent/pattern lookup index |
| `rvf-quant` | `rlmx-rvf` | 4-bit quantized vectors for phone (8x memory savings) |
| `rvf-crypto` | `rlmx-rvf` | Ed25519 signatures, replaces HMAC stubs |

### Phase 2: Consensus & Networking (`ruvnet-phase2` feature)
| Ruvnet Crate | RLMX Target | Replaces |
|---|---|---|
| `ruvector-raft` | `rlmx-swarm/src/consensus.rs` | Stub RaftLayer → production Raft |
| `ruv-swarm-core` | `rlmx-swarm` | Stub node discovery → gossip-based mesh join |
| `ruv-swarm-transport` | `rlmx-swarm` | HTTP-only → QUIC + WebSocket + HTTP transport abstraction |
| `midstreamer-quic` | `rlmx-swarm` | No LAN transport → sub-ms QUIC on LAN |
| `qudag-dag` | `rlmx-swarm` | No DAG consensus → multi-agent DAG coordination |
| `qudag-network` | `rlmx-swarm` | Cross-device agent communication layer |
| `qudag-crypto` | `rlmx-swarm` | Cryptographic proofs for coordination audit trail |

### Phase 3: Inference & Attention (`ruvnet-phase3` feature)
| Ruvnet Crate | RLMX Target | Replaces |
|---|---|---|
| `ruvector-tiny-dancer-core` | `rlmx-kernel/src/router.rs` | Stub TinyDancerRouter → production FastGRNN |
| `ruvector-sona` | `rlmx-cognitive/src/sona.rs` | Stub Sona → production SONA with real micro-LoRA |
| `ruvector-attention` | `rlmx-ruvllm` | Standard attention → Flash Attention (7.4x speedup) |
| `ruvector-attn-mincut` | `rlmx-ruvllm` | No attention pruning → min-cut attention (60% token reduction) |
| `ruvector-solver` | `rlmx-cognitive` | No LP solver → linear programming for budget optimization |
| `ruvector-gnn` | `rlmx-cognitive` | No GNN → graph neural network over life graph |
| `cognitum-gate-tilezero` | `rlmx-ruvllm` | No sparse attention → tile-zero gated attention |
| `cognitum-gate-kernel` | `rlmx-ruvllm` | Standard kernels → fused GPU kernels (30% faster) |
| `midstreamer-scheduler` | `rlmx-kernel/src/scheduler.rs` | Basic scheduler → cross-device task scheduling |

### Phase 4: Neural & Edge (`ruvnet-phase4` feature)
| Ruvnet Crate | RLMX Target | Replaces |
|---|---|---|
| `ruvector-cognitive-container` | `rlmx-swarm/src/sandbox.rs` | SandboxProfile → cognitive containers with isolated SONA |
| `ruvector-memopt` | `rlmx-ruvllm` | No mobile optimization → gradient checkpointing, memory-mapped vectors |
| `ruv-neural-core` | All inference crates | Per-platform matmul → unified Metal/WASM SIMD/NEON/AVX-512 |
| `ruv-neural-signal` | `rlmx-phone` | No signal processing → IoT anomaly detection |
| `ruv-neural-graph` | `rlmx-cognitive` | No message passing → GNN message passing over life graph |
| `ruv-neural-embed` | All crates | Per-crate embedding → unified 768-dim embedding generation |
| `ruv-neural-memory` | `rlmx-cognitive` | No episodic memory → differentiable episodic memory |
| `cognitum-rs` | `rlmx-cognitive` | Separate SONA/DagOpt/NervousSystem → unified cognitive framework |

### Phase 5: Bare-Metal & Verification (`ruvnet-phase5` feature)
| Ruvnet Crate | RLMX Target | Replaces |
|---|---|---|
| `ruvix-cap` | `rlmx-kernel` | HMAC-SHA256 tokens → Ed25519 + Macaroon capability tokens |
| `ruvix-hal` | `rlmx-phone` / new | No HAL → hardware abstraction for Pi GPIO |
| `ruvix-bcm2711` | New | No Pi drivers → BCM2711 I2C/SPI drivers |
| `ruvix-region` | `rlmx-kernel/src/memory.rs` | Logical isolation → hardware-enforced memory regions |
| `ruvix-nucleus` | New | No bare-metal → minimal kernel for Pi overnight mode |
| `ruvix-sched` | New | No cooperative scheduler → interrupt-driven bare-metal scheduler |
| `ruvix-types` | `rlmx-kernel/src/types.rs` | Shared types alignment across full stack |
| `ruvector-verified` | `rlmx-kernel` | No formal verification → proven token narrowing + witness chain integrity |

### Integration Pattern (All Phases)
```rust
// In Cargo.toml
[dependencies]
ruvector-core = { version = "0.1", optional = true }

[features]
ruvnet-phase1 = ["ruvector-core", "ruvector-graph", ...]

// In source code
#[cfg(feature = "ruvnet-phase1")]
use ruvector_core::HnswIndex;

#[cfg(not(feature = "ruvnet-phase1"))]
mod stub_hnsw { /* existing brute-force implementation */ }
```

### Build Verification
```bash
cargo build --workspace                           # Stub build — MUST pass
cargo build --workspace --features ruvnet-phase1  # Phase 1 integration
cargo build --workspace --all-features            # Full build with all phases
cargo test --workspace                            # All 695+ tests pass
cargo clippy --workspace -- -D warnings           # Zero warnings
```

## Consequences

### Positive
- Stub build always works — no ruvnet dependency required for development
- Each phase independently testable and deployable
- Existing tests continue to pass against stubs
- Production deployments opt into specific phases based on target device
- Clear upgrade path from stubs to production implementations

### Negative
- Feature gate complexity — conditional compilation across many crates
- Potential for feature interaction bugs when multiple phases enabled
- Maintaining both stub and production code paths until stubs are deprecated

### Risks
- Ruvnet crate API changes breaking integration (mitigated: pin versions, integration tests)
- Feature gate combinatorial explosion (mitigated: phases are sequential, not combinatorial)

## References
- PRD: "RuVix Mesh", Implementation Plan Steps 3-6
- ADR-004: Tiered Model Inference (existing feature gate pattern)
- ADR-007: RuVix Kernel Migration (kernel type system)
