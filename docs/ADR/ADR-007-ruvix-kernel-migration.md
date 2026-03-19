# ADR-007: Progressive Migration to RuVector Ecosystem Crates

## Status
Proposed

## Date
2026-03-18

## Context
The RLMX kernel (`rlmx-kernel`) contains custom implementations of several
subsystems: HMAC-SHA256 capability tokens, cosine-similarity vector search,
Karger/Stoer-Wagner min-cut, heat-kernel graph diffusion, SONA micro-LoRA
adaptation, DAG optimization, and a bio-inspired nervous system. These were
necessary during initial development but now have mature, battle-tested
equivalents in the RuVector ecosystem crates (`ruvix-cap`, `ruvector-sona`,
`ruvector-dag`, `ruvector-nervous-system`, `ruvector-attention`,
`ruvector-mincut`, `ruvix-types`).

Maintaining parallel implementations increases the maintenance burden and
diverges from upstream improvements. The 25-node swarm deployment amplifies
this -- bugs in custom implementations affect all nodes simultaneously.

Phase 2 (Weeks 5-8) of the swarm buildout is the natural migration window:
the kernel API is stable, the swarm orchestration layer sits above it, and
we can migrate one subsystem at a time without blocking other work.

## Decision
Progressively replace four kernel subsystems with thin wrappers around
RuVector ecosystem crates, preserving all existing public APIs.

### Migration 1: Capability System

**Current**: `crates/rlmx-kernel/src/capability.rs` -- custom `CapabilityToken`
with HMAC-SHA256 signing, TTL expiry, hierarchical derivation. Dependencies:
`hmac`, `sha2`.

**Target**: Wrap `ruvix_cap::Capability` and `ruvix_cap::CapabilitySpace`.

```rust
// crates/rlmx-kernel/src/capability.rs (after migration)
use ruvix_cap::{Capability, CapabilitySpace};

pub struct CapabilityToken {
    inner: Capability,
    space: Arc<CapabilitySpace>,
}

impl CapabilityToken {
    pub fn derive(&self, restricted_permissions: &[SyscallPermission]) -> Result<Self> {
        let child = self.space.derive(&self.inner, restricted_permissions)?;
        Ok(Self { inner: child, space: self.space.clone() })
    }
}
```

Removes direct `hmac`/`sha2` dependencies from `rlmx-kernel`. The
`CapabilityToken` public API (`new()`, `derive()`, `verify()`, `has_permission()`,
`is_expired()`) remains identical.

### Migration 2: Cognitive Layer

**Current**: `crates/rlmx-cognitive/src/` -- `sona.rs` (micro-LoRA + EWC++),
`dag.rs` (DAG optimizer), `nervous.rs` (BTSP, HDC, WTA, circadian, global
workspace), `attention.rs` (attention selection).

**Target**: Thin wrappers around `ruvector-sona`, `ruvector-dag`,
`ruvector-nervous-system`, `ruvector-attention`.

```rust
// crates/rlmx-cognitive/src/sona.rs (after migration)
use ruvector_sona::{SonaEngine, PatternBank, MicroLoraAdapter};

pub struct Sona {
    engine: SonaEngine,
}

impl Sona {
    pub fn adapt(&mut self, input: &Tensor, target: &Tensor) -> Result<AdaptResult> {
        self.engine.adapt(input, target).map(Into::into)
    }
    pub fn store_pattern(&mut self, pattern: Pattern) -> Result<PatternId> {
        self.engine.pattern_bank_mut().store(pattern.into())
    }
}
```

Removes `ndarray` dependency from `rlmx-cognitive`. The `ruvector-*` crates
use their own tensor types; bridge via `From`/`Into` conversions in
`crates/rlmx-cognitive/src/bridge.rs`.

### Migration 3: Graph Min-Cut

**Current**: `crates/rlmx-kernel/src/graph.rs` -- in-memory property graph with
Karger randomized min-cut and Stoer-Wagner deterministic min-cut.

**Target**: Keep the `Graph` struct, `CypherParser`, edge/node storage, and
`GraphQuery`/`GraphDiffuse` handlers. Replace only the min-cut algorithms
with calls to `ruvector_mincut::{karger, stoer_wagner}`.

```rust
// In graph.rs dispatch for GraphCut syscall
use ruvector_mincut::{karger, stoer_wagner};

fn handle_graph_cut(&self, algorithm: CutAlgorithm) -> Result<CutResult> {
    let adjacency = self.to_adjacency_matrix();
    match algorithm {
        CutAlgorithm::Karger => karger(&adjacency, self.rng()),
        CutAlgorithm::StoerWagner => stoer_wagner(&adjacency),
    }
}
```

The Cypher parser and property graph storage remain custom -- no RuVector
equivalent exists for these.

### Migration 4: Type Bridge

**Current**: `crates/rlmx-kernel/src/types.rs` -- custom `VectorId`,
`GraphNode`, `ProcessId`, `CapabilityId`, etc.

**Target**: Add `From`/`Into` implementations between RLMX types and
`ruvix_types` equivalents. Both type sets coexist during migration; downstream
crates gradually adopt `ruvix_types` directly.

```rust
// crates/rlmx-kernel/src/types.rs
impl From<ruvix_types::VectorId> for VectorId {
    fn from(v: ruvix_types::VectorId) -> Self {
        Self(v.as_u128())
    }
}
impl From<VectorId> for ruvix_types::VectorId {
    fn from(v: VectorId) -> Self {
        ruvix_types::VectorId::from_u128(v.0)
    }
}
```

### Migration Order

| Week | Subsystem | Crate Dependency Added | Dependency Removed |
|------|-----------|----------------------|-------------------|
| 5 | Capability | `ruvix-cap` | `hmac`, `sha2` |
| 6 | Cognitive (SONA, DAG) | `ruvector-sona`, `ruvector-dag` | `ndarray` |
| 7 | Cognitive (Nervous, Attention) | `ruvector-nervous-system`, `ruvector-attention` | -- |
| 8 | Graph min-cut + Type bridge | `ruvector-mincut`, `ruvix-types` | -- |

Each migration is a single PR with:
1. Add new dependency to `Cargo.toml`.
2. Replace implementation, keep public API.
3. Update tests to verify identical behavior.
4. `cargo test --workspace` must pass all 134+ tests.

## Consequences

### Positive
- Reduced maintenance burden -- upstream crates handle bug fixes and optimizations.
- Smaller dependency tree after removing `hmac`, `sha2`, `ndarray` direct deps.
- Access to upstream performance improvements (e.g., SIMD-optimized min-cut).
- Consistent type system across RLMX and other RuVector-based projects.
- Each migration is independently revertible -- wrapper pattern isolates changes.

### Negative
- New external dependencies introduce supply chain risk (mitigated by pinning
  versions and auditing via `cargo-audit`).
- Thin wrapper overhead is negligible but nonzero (one extra function call layer).
- Two type systems coexist during migration period (Weeks 5-8), increasing
  cognitive load for developers.

### Risks
- RuVector crate APIs may not be 100% compatible with RLMX semantics. Mitigated
  by the wrapper pattern -- any gap is handled in the bridge layer.
- Upstream breaking changes could block migration. Mitigated by pinning to
  specific versions in `Cargo.toml`.
- Migration could introduce subtle behavioral differences (e.g., different RNG
  in Karger min-cut). Mitigated by property-based tests comparing old vs new.

## Alternatives Considered

1. **Keep custom implementations**: Zero migration risk, but ongoing maintenance
   cost grows linearly with codebase. The 25-node swarm makes bugs more costly.
   Rejected.

2. **Full rewrite to RuVector types everywhere**: Would require changing every
   callsite across all 9 crates simultaneously. Too risky for a single migration.
   Rejected in favor of progressive approach.

3. **Gradual deprecation (keep both, let callers choose)**: Creates API
   confusion and doubles the test surface. Rejected.

## References
- `crates/rlmx-kernel/src/capability.rs` -- current capability implementation.
- `crates/rlmx-kernel/src/graph.rs` -- current graph + min-cut implementation.
- `crates/rlmx-cognitive/src/` -- current cognitive layer.
- RuVector ecosystem: https://github.com/ruvector
- `ruvix-cap` crate documentation.
- `ruvector-sona` crate documentation.
