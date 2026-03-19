# Claude Code Configuration — RLMX Cognition Kernel

## Behavioral Rules (Always Enforced)

- Do what has been asked; nothing more, nothing less
- NEVER create files unless absolutely necessary
- ALWAYS prefer editing existing files over creating new ones
- NEVER proactively create documentation files (*.md) or README files unless explicitly requested
- NEVER save working files, text/mds, or tests to the root folder
- Never continuously check status after spawning a swarm — wait for results
- ALWAYS read a file before editing it
- NEVER commit secrets, credentials, or .env files

## File Organization

- Root folder: NEVER save files here
- `/crates` — Rust source (each crate has own `src/`)
- `/docs` — documentation, ADRs, DDD documents (all `.md` files go here)
- `/frontend` — web UI (single-page `index.html` — do NOT split)
- `/mobile` — React Native app (TypeScript, Android/iOS)
- `/deploy` — deployment configs
- `/scripts` — utility and hook scripts
- `/packages` — TypeScript @aix packages

## Project Overview

RLMX ("RuVix") is a **voice-first cognition kernel** — an OS-kernel-inspired runtime for LLM agents. Rust (edition 2021) + async Tokio + React Native mobile.

**22 Rust crates + 20 npm packages (@aix), 82 MCP tools, 17 agent types, 6 swarm zones, 1,286+ Rust tests + 1,135 TS tests (2,421+ total), 39 ADRs, 16 DDD bounded contexts.**

## Quick Build Reference

```bash
cargo build --workspace                              # Build all
cargo test --workspace                               # Test all (1,286+)
cargo clippy --workspace -- -D warnings              # Lint (zero warnings)
npm run test:ts                                      # TS tests (1,135)
cargo run -p rlmx-cli -- serve --port 3000           # MCP server
```

Full commands: **[docs/BUILD-COMMANDS.md](docs/BUILD-COMMANDS.md)**

## Conventions

**Rust:**
- `Arc<Mutex<T>>` / `Arc<RwLock<T>>` for shared state
- `thiserror` for per-crate errors; `tracing` for logging (no `println!`)
- Tests: inline `#[cfg(test)]` blocks
- Feature gates follow ADR-024 (`ruvnet-phase1..5`), always with stub fallback
- Aggregate roots own consistency boundaries; cross-context via domain events

**Type ownership (never duplicate):**
- `rlmx-kernel`: `SyscallPermission`, `Strategy`, `ProcessId`, `LifeDomain`, `Intent`, `ResponseMode`, `VoicePersona`
- `rlmx-mesh`: `MeshId`, `DeviceId`, `DeviceZone`, `MeshDevice`
- `rlmx-federation`: `FederationCycle`, `Contribution`, `FederationPackage`
- `rlmx-billing`: `SubscriptionTier`, `TierLimits`, `FamilyGroup`, `BudgetLedger`, `BudgetPolicy`
- `rlmx-artifact`: `ArtifactId`, `Artifact`, `ArtifactDiff`, `ContentAddressedStore`, `BranchManager`
- `rlmx-evolve`: `FunctionId`, `EvolvedFunction`, `FunctionStatus`, `ScoreResult`, `FeedbackDecision`
- `rlmx-channels`: `ChannelAdapter`, `ChannelMessage`, `ChannelRegistry`, `ChannelId`
- `rlmx-kernel` (new modules): `ApprovalTier`, `ApprovalGate`, `TriggerBinding`, `TriggerRegistry`, `AgentCard`, `A2ASkill`
- `packages/deploy`: Deploy manifest types, bridge adapters (domain-specific, not in kernel/shared)
- New crate domain events: crate-local enums, not kernel `DomainEvent`

**TypeScript** (details in `docs/TYPESCRIPT-MIGRATION.md`):
- Build: tsup (CJS+ESM+.d.ts), test: vitest, workspace: npm workspaces
- Import types from `@aix/shared` — never re-define kernel enums
- `generateId()` from `@aix/shared`; `Object.setPrototypeOf` in error classes
- All packages handle `@aix/core` unavailability (`{ status: 'unavailable' }`)
- Templates are frozen (`Object.freeze`) — clone before mutation
- `DeployError` extends `AixError` (-36xxx code range) — ADR-029

## Strict Rules

1. **Stub build**: `cargo build --workspace` without features MUST compile clean
2. **No println!**: use `tracing::*` in library crates
3. **RBAC**: Admin/System only via server-side `token_roles`
4. **Tests pass**: `cargo test --workspace` >= 1,286; `npm run test:ts` >= 1,135
5. **Zero clippy warnings**: `-D warnings`
6. **Edge unavailable**: tools return `"status": "unavailable"` without engine
7. **GGUF tokenizers**: `<model>-tokenizer.json` required next to `.gguf`
8. **MCP handshake**: initialization handshake mandatory
9. **Permission matrix**: `PermissionRegistry` in `registry.rs` (18x17) is source of truth — 18 permissions (incl. ArtifactWrite) x 17 agent types
10. **Domain events flow**: dispatch() emits SyscallDispatched, voice sessions emit VoiceSessionStarted
11. **Frontend fallback**: check empty data (`node_count === 0`), not just catch errors
12. **Single-file frontend**: `frontend/index.html` — never split
13. **Voice privacy**: no audio leaves device; STT on-device; only text to SONA
14. **Federation privacy**: Laplace e=1.0, 1000-user min, emotion bucketed (5 levels), no speaker embeddings
15. **Phone engagement**: LifeScore floor 30, ProofSeal for savings, streak freeze max 1/30d, offline queue max 100
16. **Marketplace security**: automated review required, 70/30 split enforced
17. **Mobile offline**: demo mode with realistic data when no server
18. **Android JDK 21**: `JAVA_HOME=/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home`
19. **Phase gates**: both `#[cfg(feature)]` and `#[cfg(not(feature))]` paths required
20. **New crates**: add to Cargo.toml members, update counts, add to key files
21. **New agent types**: update AgentType enum + PermissionRegistry + spawn hierarchy + CLI + MCP
22. **Billing enforcement**: Free=5 agents via capability tokens; never bypass tier checks
23. **Mesh privacy anchor**: home hub (Zone C/E) sole long-term store; max 1 MeshCoordinator
24. **Use case compliance**: UC1 (585 tests), UC2 (279 tests) must pass
25. **SwarmEvent sync**: `ws.rs` and `types.rs` variants must match
26. **Cross-device types**: shared between napi/wasm must live in kernel
27. **LifeDomain closed**: 12 variants fixed (Finance, Health, Legal, Career, Education, Home, Shopping, Travel, Social, Government, Automotive, Pet)
28. **Engagement caps**: levels max 10 (50 XP each), achievements max 50, LifeScore weights sum to 1.0
29. **TS typecheck**: `npx tsc --noEmit` must pass for all 20 packages
30. **@aix/shared types**: never duplicate in TS packages
31. **@aix/core degradation**: mandatory `{ status: 'unavailable' }` handling
32. **Artifact DAG**: content-addressed SHA-256; parent validation on insert; 512KB max per artifact
33. **Approval tiers**: Auto/Notify/Confirm/Escalate; Confirm/Escalate auto-deny after 5min timeout
34. **Trigger depth**: max 5 recursive trigger invocations to prevent cycles
35. **Evolution lifecycle**: draft→staging→production→deprecated→killed; no skip transitions; dual-gate (safety+sandbox) required for staging
36. **Budget CAS**: BudgetLedger uses compare-and-swap versioning; reject stale concurrent writes
37. **Channel secrets**: Telegram/WhatsApp/Teams/Discord tokens via env vars or vault; never in code/git
38. **Score formula**: overall = correctness×0.50 + safety×0.25 + latency_score×0.15 + cost_score×0.10
39. **Board limits**: 1,000 posts/board, 25 pins/board
40. **A2A task eviction**: FIFO at 1,000 tasks; SSRF protection on remote agent URLs

## Use Case Validation

- **UC1** "One Voice, Millions of Agents" (`docs/crazy-ruv-cartes-plan.md`) — 585 tests across voice/phone/marketplace/kernel/agents/swarm/mcp
- **UC2** "Personal Agent Cloud" (`docs/crazy-ruv-cartes-plan copy.md`) — 279 tests across napi/wasm/mesh/federation/billing/cognitive
- Run UC1 suite before merging voice/phone/marketplace changes
- Run UC2 suite before merging mesh/federation/billing changes
- New agent types must map to existing 12 LifeDomains
- New MCP tools must be documented here and in `tools.rs`

## Security Rules

- NEVER hardcode API keys, secrets, or credentials
- NEVER commit .env files
- Validate user input at system boundaries
- Sanitize file paths (directory traversal)
- Voice audio NEVER leaves device
- Federated patterns: differential privacy (Laplace e=1.0)
- Marketplace agents: automated security review required

## Mobile Development

- TypeScript, functional components, React hooks
- Dark theme tokens: bg=#0a0e1a, surface=#141824, cyan=#00e5ff, violet=#a855f7, green=#22c55e, amber=#f59e0b, red=#ef4444, pink=#ec4899
- Must work in demo mode AND live mode
- React Navigation (bottom tabs + stack)
- APK: `mobile/android/app/build/outputs/apk/debug/app-debug.apk`

## Dashboard Development

- Keep `frontend/index.html` under 3000 lines
- Views go in `viewRenderers` as `'section.view': { desc, render, onShow }`
- Fallback pattern: try server -> check if empty -> use DemoData
- Cancel `requestAnimationFrame` on view switch
- WASM: `await mod.default()` first, never `mod.init()`

## Concurrency

- All operations MUST be concurrent/parallel in a single message
- Batch ALL file reads/writes/edits in ONE message
- Batch ALL Bash commands in ONE message
- Agent Tasks: `run_in_background: true`, all in ONE message
- After spawning agents: STOP and wait for results

## Environment Variables

| Variable | Default |
|----------|---------|
| `RLMX_EDGE_MODEL` | unset |
| `RLMX_MODEL_DIR` | `~/.rlmx/models` |
| `JAVA_HOME` | system |
| `RLMX_MESH_PORT` | `5353` |
| `RLMX_FEDERATION_ENDPOINT` | unset |
| `RLMX_BILLING_STRIPE_KEY` | unset |

## Hook Scripts (`scripts/hooks/`)

| Hook | Trigger |
|------|---------|
| `pre-voice.sh` | Before voice (check battery/network) |
| `post-voice.sh` | After voice (log, engagement, streak) |
| `on-savings.sh` | Savings discovered (annual impact) |
| `on-streak.sh` | Streak milestone (reward tier) |
| `on-agent-level.sh` | Agent levels up (unlock capabilities) |

## Detailed Reference (read on demand)

| Document | Contents |
|----------|----------|
| [docs/BUILD-COMMANDS.md](docs/BUILD-COMMANDS.md) | All build, test, deploy, and CLI commands |
| [docs/FEATURE-GATES.md](docs/FEATURE-GATES.md) | Feature gate matrix (13 features) and rules |
| [docs/WORKSPACE-STRUCTURE.md](docs/WORKSPACE-STRUCTURE.md) | 19 crate tree, 13 TS packages, directories, 16 DDD contexts |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Detailed component descriptions for all subsystems |
| [docs/KEY-FILES.md](docs/KEY-FILES.md) | Key file locations across all crates and packages |
| [docs/TYPESCRIPT-MIGRATION.md](docs/TYPESCRIPT-MIGRATION.md) | Full TS migration reference (packages, NAPI, DDD) |
| [docs/ADR/](docs/ADR/) | 39 Architecture Decision Records (ADR-001 through ADR-039) |
| [docs/DDD/](docs/DDD/) | 16 Domain-Driven Design documents |

## Known Gaps (2026-03-19)

| Gap | Severity |
|-----|----------|
| Phase1 feature gate: additive exports (no explicit `not(feature)` fallback) | Low — stub build unaffected |
| Mesh/federation/billing MCP tools: stub implementations | Low — JSON-RPC wired, handlers return stub data |
| Channel adapters (Telegram/WhatsApp/Teams/Discord): stub send/receive | Medium — trait + registry wired, real API calls not yet implemented |
| Evolution WASM sandbox integration: not yet wired to SandboxManager | Low — lifecycle/scoring/feedback complete, sandbox gate is placeholder |
| A2A HTTP server: types + task store ready, HTTP listener not yet wired | Low — JSON-RPC handler works, needs HTTP mount point |
| 35 new MCP tools: type-safe but not wired to rlmx-mcp tool dispatch | Medium — handlers need adding to tools.rs |

## Swarm Configuration

- Hierarchical topology for coding swarms
- maxAgents: 6-12; specialized strategy; `raft` consensus
- Phone=Zone A-Mobile (primary), Laptop=Zone A-Desktop (secondary)

## Deployment Targets

| Target | Method | Status |
|--------|--------|--------|
| Android phone | React Native APK | Working |
| Local Mac | BroadcastChannel | Working |
| RPi5/NUC | Cross-compile + systemd | Config in `deploy/` |
| Cloud GPU | SkyPilot | Planned (ADR-006) |
| Browser workers | WASM compute pool | Working (ADR-009) |
