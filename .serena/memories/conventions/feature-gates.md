# Feature Gate Rules (ADR-024)

Feature gates follow the `ruvnet-phase1` through `ruvnet-phase5` naming convention.

## Rules

1. Both `#[cfg(feature = "ruvnet-phaseN")]` AND `#[cfg(not(feature = "ruvnet-phaseN"))]` paths are required
2. The `not(feature)` path must provide a working stub/fallback
3. `cargo build --workspace` without any features MUST compile clean (stub build)
4. Features are additive — enabling a feature adds capability, never removes it
5. Each phase gate has its own Cargo.toml feature entry

## Pattern

```rust
#[cfg(feature = "ruvnet-phase1")]
pub fn real_implementation() { /* full logic */ }

#[cfg(not(feature = "ruvnet-phase1"))]
pub fn real_implementation() { /* stub fallback */ }
```
