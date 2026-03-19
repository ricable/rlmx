# ADR-026: TypeScript Migration Strategy

Status: Proposed

## Context
RLMX currently consists of 19 Rust crates (~49K LOC). Analysis shows 57% of the codebase is orchestration and business logic (agents, swarm coordination, marketplace CRUD, billing rules, CLI wiring) better served by a higher-level language with faster iteration cycles. The remaining 34% is compute-heavy code (neural nets, vector math, cryptographic proofs, voice signal processing) that benefits from Rust's performance and safety guarantees.

The project needs to evolve from a monolithic Rust workspace into a hybrid architecture that:
- Preserves Rust for compute-critical paths (kernel, cognitive, voice, phone, inference)
- Rewrites orchestration layers in TypeScript for faster feature development
- Packages the result as `npx aix` following the ruvector distribution pattern
- Maintains all existing invariants (privacy, capability security, tier enforcement)

## Decision
Migrate 10 of 19 Rust crates to TypeScript as `@aix/*` npm packages. The remaining 9 Rust crates are compiled into a single NAPI binary (`@aix/core`) and a WASM module (`@aix/wasm`). The migration proceeds in 5 phases, with each phase building on the previous.

### Crate Classification

| Classification | Crates | Rationale |
|----------------|--------|-----------|
| **Stay Rust** (compiled into `@aix/core`) | rlmx-kernel, rlmx-cognitive, rlmx-ruvllm, rlmx-trm, rlmx-rvf, rlmx-voice, rlmx-phone, rlmx-napi | Performance-critical: vector math, neural nets, VAD/STT/TTS, engagement scoring, Ed25519 proofs, Candle inference |
| **Stay Rust** (compiled to WASM) | rlmx-wasm | Browser kernel subset — already WASM-targeted |
| **Migrate to TypeScript** | rlmx-cli, rlmx-mcp, rlmx-agents, rlmx-swarm, rlmx-marketplace, rlmx-mesh, rlmx-billing, rlmx-federation, rlmx-plugin, rlmx-rlm | Orchestration, CRUD, HTTP/WS servers, CLI wiring — no compute-intensive paths |

### Migration Phases

| Phase | Packages | Dependencies | LOC Ported |
|-------|----------|--------------|------------|
| 1 — Foundation | `@aix/shared`, `@aix/core`, `@aix/wasm` | None (leaf packages) | Types + NAPI expansion |
| 2 — Leaf | `@aix/rlm`, `@aix/plugin`, `@aix/billing` | `@aix/shared`, `@aix/core` | ~5,400 |
| 3 — Mid-level | `@aix/agents`, `@aix/mesh`, `@aix/federation`, `@aix/marketplace` | `@aix/shared`, `@aix/core` | ~11,200 |
| 4 — Composition | `@aix/swarm`, `@aix/mcp-server` | `@aix/agents`, `@aix/core`, `@aix/shared` | ~11,200 |
| 5 — CLI | `aix` | All `@aix/*` packages | ~2,400 |

### Archive Strategy
When a TypeScript package passes its ported test suite:
1. Move `crates/rlmx-<name>/` → `crates/archive/rlmx-<name>/`
2. Remove from `Cargo.toml` workspace `members`
3. Verify `cargo build --workspace` compiles clean with fewer members
4. Archived crates remain as reference — not compiled, not tested

### Post-Migration Workspace

**Rust (9 crates)**:
```
rlmx-kernel, rlmx-cognitive, rlmx-ruvllm, rlmx-trm, rlmx-rvf,
rlmx-voice, rlmx-phone, rlmx-napi, rlmx-wasm
```

**TypeScript (13 npm packages)**:
```
aix, @aix/core, @aix/wasm, @aix/shared, @aix/mcp-server,
@aix/agents, @aix/marketplace, @aix/mesh, @aix/billing,
@aix/swarm, @aix/federation, @aix/plugin, @aix/rlm
```

### Build Tooling

| Concern | Tool | Rationale |
|---------|------|-----------|
| TS compilation | tsup (esbuild-based) | CJS + ESM + .d.ts in one pass |
| Testing | vitest | Fast, ESM-native, compatible with tsup output |
| Workspace | npm workspaces | Built-in, no extra tooling |
| NAPI | napi-rs | Already used by rlmx-napi (ADR-020) |
| WASM | wasm-pack | Already used by rlmx-wasm (ADR-021) |

### Graceful Degradation
All TypeScript packages must handle the case where `@aix/core` fails to load (e.g., unsupported platform, missing binary):
```typescript
try {
  const core = require('@aix/core');
  return core.napiDispatch(syscall, args);
} catch {
  return { status: 'unavailable', reason: 'native module not loaded' };
}
```
This mirrors the existing Rust pattern where edge tools return `"status": "unavailable"` when no engine is configured.

## Consequences

### Positive
- 57% of codebase moves to TypeScript — faster iteration, broader contributor pool
- `npx aix` distribution — zero-config install for users
- Per-platform NAPI binaries — no compilation required at install time
- Existing privacy/security invariants preserved — compute stays in Rust
- MCP server can use official `@modelcontextprotocol/sdk` instead of custom JSON-RPC
- Vitest enables faster test cycles than `cargo test` for orchestration logic

### Negative
- Dual-language codebase requires expertise in both Rust and TypeScript
- NAPI bridge adds FFI overhead for cross-boundary calls (mitigated: sub-millisecond per ADR-020)
- Archive strategy means 10 crates become read-only reference — no backports
- Type definitions must be kept in sync between `@aix/shared` and Rust kernel types

### Risks
- NAPI bridge expansion (6 → ~31 functions) may surface new FFI edge cases (mitigated: ADR-027 details testing strategy)
- Test count will temporarily decrease as Rust tests are ported to vitest (mitigated: port tests before archiving crate)
- Breaking change for any existing Rust-only integrations (mitigated: `@aix/core` exposes same kernel operations)

## References
- ADR-020: NAPI-RS Native Bindings (existing NAPI bridge)
- ADR-021: WASM Kernel Subset (existing WASM module)
- ADR-024: Ruvnet Crate Integration (feature gates preserved in remaining Rust crates)
- ADR-027: NAPI Bridge Expansion (detailed function list)
- ADR-028: @aix Package Architecture (npm workspace layout)
- DDD-014: NAPI Core Bridge Context
- DDD-015: TypeScript Package Context Map
