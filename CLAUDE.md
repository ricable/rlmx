# Claude Code Configuration — RLMX Cognition Kernel

## Behavioral Rules (Always Enforced)

- Do what has been asked; nothing more, nothing less
- NEVER create files unless they're absolutely necessary for achieving your goal
- ALWAYS prefer editing an existing file to creating a new one
- NEVER proactively create documentation files (*.md) or README files unless explicitly requested
- NEVER save working files, text/mds, or tests to the root folder
- Never continuously check status after spawning a swarm — wait for results
- ALWAYS read a file before editing it
- NEVER commit secrets, credentials, or .env files
- ALWAYS run `cargo fmt` after writing Rust code — agents generate unformatted code
- NEVER add a new crate without updating this file (Workspace Structure, Architecture, Key File Locations, DDD, ADR Index)
- ALWAYS verify the full pipeline after changes: `cargo fmt && cargo build --workspace && cargo test --workspace && cargo clippy --workspace -- -D warnings`

## File Organization

- NEVER save to root folder — use the directories below
- Use `/crates` for Rust source code (each crate has its own `src/`)
- Use `/crates/archive` for deprecated/legacy crate code
- Use `/docs` for documentation, ADRs, and DDD documents
- Use `/frontend` for web UI files (single-page `index.html` — do NOT split into multiple files)
- Use `/mobile` for React Native mobile app (TypeScript, Android/iOS)
- Use `/deploy` for deployment configs
- Use `/scripts` for utility scripts and hook scripts
- Documentation changes go in `/docs` — do NOT create `.md` files in other directories

## Project Overview

RLMX ("RuVix") is a **voice-first cognition kernel** — an OS-kernel-inspired runtime for LLM agents, activated by voice. It provides capability-secured syscall primitives that agents call instead of accessing arbitrary APIs. Written in Rust (edition 2021), async on Tokio, with a React Native mobile app for Android/iOS.

**19 crates, 39 MCP tools, 15 agent types, 6 swarm zones, 925+ tests, 20+ dashboard views, 11 sandbox profiles, 25 ADRs, 13 DDD bounded contexts.**

The system supports seven interaction paths:
- **Voice-First**: On-device STT (Whisper-tiny Q4) → TinyDancerRouter (18-dim) → Multi-Intent Decomposition → Agent Swarm → Multimodal Response (voice + cards + haptics)
- **Cloud/Dev**: `VllmClient` → vLLM on Metal (Mac) or NVIDIA GPU (prod)
- **Edge**: `LocalEngine` → ruvllm CandleBackend on CPU/Metal/CUDA with GGUF models
- **Tiered**: `TieredEngine` → Small (0.5B) → Medium (MLX 3-8B) → Remote (vLLM) with confidence escalation
- **Browser**: `@ruvector/ruvllm-wasm` → WebGPU/WASM SIMD (inference primitives + HTTP fallback)
- **NAPI/Node.js**: `rlmx-napi` → Native Node.js addon exposing full kernel syscall surface
- **Swarm**: Cross-zone scatter-gather via `Strategy::Swarm` with configurable gather strategy

## Build & Development Commands

```bash
# === Standard Development ===
cargo build --workspace              # Build all 19 crates
cargo test --workspace               # Run all tests (925+)
cargo test -p rlmx-kernel            # Run tests for a single crate
cargo clippy --workspace -- -D warnings  # Lint (must be zero warnings)
cargo fmt --check                    # Check formatting
cargo fmt                            # Format code

# === Full Verification Pipeline (run after any change) ===
cargo fmt && cargo build --workspace && cargo test --workspace && cargo clippy --workspace -- -D warnings

# === Run the MCP Server ===
cargo run -p rlmx-cli -- serve --port 3000      # Start MCP + WS server

# === Voice Commands (CLI) ===
cargo run -p rlmx-cli -- voice start                              # Start voice pipeline
cargo run -p rlmx-cli -- voice transcribe --text "hello world"    # Simulate STT
cargo run -p rlmx-cli -- voice intents --text "I want to move to Paris"  # Show intent decomposition
cargo run -p rlmx-cli -- voice session --list                     # List voice sessions

# === Marketplace Commands (CLI) ===
cargo run -p rlmx-cli -- marketplace search --domain finance      # Search agents by domain
cargo run -p rlmx-cli -- marketplace install --agent bill-negotiator  # Install agent
cargo run -p rlmx-cli -- marketplace list                         # List installed agents
cargo run -p rlmx-cli -- marketplace featured                     # Show featured agents
cargo run -p rlmx-cli -- marketplace publish --path ./agent.rvf   # Publish agent

# === Engagement Commands (CLI) ===
cargo run -p rlmx-cli -- engagement score                         # Show Life Score (0-100)
cargo run -p rlmx-cli -- engagement savings                       # Show Money Saved counter
cargo run -p rlmx-cli -- engagement streak                        # Show current streak
cargo run -p rlmx-cli -- engagement achievements                  # List achievements

# === Phone Runtime Commands (CLI) ===
cargo run -p rlmx-cli -- phone status                             # Phone runtime status
cargo run -p rlmx-cli -- phone agents                             # List on-device agents
cargo run -p rlmx-cli -- phone battery                            # Battery-aware scheduling info

# === Mesh Commands (CLI) — ADR-022 ===
cargo run -p rlmx-cli -- mesh status                              # Personal mesh status
cargo run -p rlmx-cli -- mesh devices                             # List mesh devices
cargo run -p rlmx-cli -- mesh add-device --name "laptop" --device-type desktop  # Register device
cargo run -p rlmx-cli -- mesh sync                                # Force cross-device sync
cargo run -p rlmx-cli -- mesh fleet                               # Fleet overview
cargo run -p rlmx-cli -- mesh failover                            # Failover & degradation status

# === Billing Commands (CLI) — ADR-025 ===
cargo run -p rlmx-cli -- billing status                           # Subscription status
cargo run -p rlmx-cli -- billing tiers                            # List all 6 tiers
cargo run -p rlmx-cli -- billing usage                            # Current period usage
cargo run -p rlmx-cli -- billing upgrade --tier pro               # Upgrade subscription
cargo run -p rlmx-cli -- billing developer                        # Developer revenue dashboard

# === Federation Commands (CLI) — ADR-023 ===
cargo run -p rlmx-cli -- federation status                        # Current cycle status
cargo run -p rlmx-cli -- federation contribute                    # Submit anonymized patterns
cargo run -p rlmx-cli -- federation history                       # Past cycle history

# === Original Commands ===
cargo run -p rlmx-cli -- query -i "search term"    # Semantic search
cargo run -p rlmx-cli -- swarm start --zone A       # Start swarm node
cargo run -p rlmx-cli -- agent spawn --type worker   # Spawn agent
cargo run -p rlmx-cli -- research start --topic "X"  # Start research
cargo run -p rlmx-cli -- sandbox spawn --profile worker  # Spawn sandbox

# === Edge Inference ===
cargo build -p rlmx-cli --features ruvllm            # CPU inference
cargo build -p rlmx-cli --features "ruvllm,metal"     # Metal GPU (macOS)
RLMX_EDGE_MODEL=tinyllama-1.1b-q4_k_m \
  cargo run -p rlmx-cli --features "ruvllm,metal" -- serve --port 3000

# === Mobile App (React Native) ===
cd mobile && npm install                              # Install dependencies
cd mobile && npx react-native start                   # Start Metro bundler
JAVA_HOME=/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home \
  cd mobile/android && ./gradlew assembleDebug        # Build Android APK (requires JDK 21)
adb install mobile/android/app/build/outputs/apk/debug/app-debug.apk  # Install on device

# === Frontend (separate terminal) ===
cd frontend && python3 -m http.server 8080            # Serve web dashboard at :8080

# === Cross-compilation (RPi5/ARM64) ===
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu -p rlmx-cli --features ruvllm
```

No Makefile, no CI pipeline. Default rustfmt and clippy settings apply.

## Workspace Structure (19 crates)

```
rlmx-cli (binary)
  ├── rlmx-kernel       (core — 15+ syscalls, router, events, types, capability, proof)
  ├── rlmx-mcp          → rlmx-kernel, rlmx-rvf, rlmx-ruvllm
  ├── rlmx-swarm        → rlmx-kernel (consensus, transport, discovery, zones)
  ├── rlmx-agents       → rlmx-kernel, rlmx-cognitive
  ├── rlmx-voice        → rlmx-kernel (voice pipeline, intent decomposition)
  ├── rlmx-phone        → rlmx-kernel (mobile runtime, engagement)
  ├── rlmx-marketplace  → rlmx-kernel, rlmx-rvf (agent marketplace)
  ├── rlmx-mesh         → rlmx-kernel (personal mesh, multi-device sync)
  ├── rlmx-billing      (standalone — subscription tiers, family plans, developer rev share)
  ├── rlmx-federation   (standalone — federated learning cycles, anonymization)
  ├── rlmx-napi         → rlmx-kernel (NAPI-RS Node.js native binding)
  ├── rlmx-wasm         (standalone — WASM kernel subset for Zone D)
  ├── rlmx-rvf          (standalone — container format)
  ├── rlmx-plugin       (standalone — domain plugins)
  └── rlmx-ruvllm       (standalone — feature-gated edge inference, phase3 neural)

rlmx-rlm               (standalone — vLLM HTTP client)
rlmx-trm               (standalone — pure numeric NN)
rlmx-cognitive          (standalone — SONA, GNN, LP solver, attention optimizer, voice patterns)
```

Additional directories:
- `crates/archive/` — Deprecated/legacy crate code
- `mobile/` — React Native mobile app (8 screens, 11 components, TypeScript)
- `frontend/` — Single-page web UI (20+ views) + WASM module
- `frontend/dashboard/` — Svelte 5 + TailwindCSS v4 dashboard (Vite, :5173)
- `deploy/` — Systemd service for RPi5
- `scripts/hooks/` — 5 preconfigured event hooks (voice, savings, streaks, agent levels)
- `docs/ADR/` — 25 Architecture Decision Records
- `docs/DDD/` — 13 Domain-Driven Design documents
- `.cargo/` — Cross-compilation config

## Architecture

### Kernel (`rlmx-kernel`)
**15-syscall dispatch** (12 original + VoiceTranscribe, VoiceSynthesize, IntentRoute) plus MeshSync and FederationContribute syscalls. `DomainEventBus` (8+ cross-context events including VoiceSessionStarted, IntentsDecomposed, MeshDeviceJoined, FederationCycleCompleted). `TinyDancerRouter` (FastGRNN **18→32→5**) with 4 voice-aware dimensions (speaker_confidence, emotion_valence, urgency_score, ambient_noise_level) and online learning with 0.6 confidence gating. `Strategy` enum includes `Swarm { scatter_zones, gather_strategy, timeout_ms }`.

**Phase 1 (Vector & Storage)**: `BloomScreenedRegion` (bloom filter pre-screening), `NamespacedMemoryStore` (per-tenant HNSW isolation), `EnhancedGraph` (ruvector-graph dual-write).

**Phase 5 (Bare-Metal & Verification)**: `MacaroonCapabilityManager` (Ed25519 + hierarchical narrowing), `Phase5ProofEngine` (3-tier routing), `IsolatedMemoryRegion` (ACL enforcement).

Types: `ResponseMode` (VoiceOnly/Visual/Multimodal/Ambient), `VoicePersona` (6 domain personas), `LifeDomain` (12 life categories), `Intent`, `MeshId`, `DeviceId`, `DeviceZone`, `SubscriptionTier`, `FederationStatus`.

### Personal Mesh (`rlmx-mesh`) — ADR-022, DDD-011
**PersonalMesh** aggregate root managing a single user's device fleet. `MeshDevice` (DeviceType, Zone, capabilities), `SyncProtocol` (state vector sync, conflict-free merging), `DiscoveryService` (mDNS/BLE/manual), `FailoverPolicy` (graceful degradation across connectivity loss). 7 source files, 57 tests.

### Federation (`rlmx-federation`) — ADR-023, DDD-012
**FederationCycle** aggregate root with 4-phase lifecycle: Collecting → Aggregating → Distributing → Completed. `Anonymizer` (PII stripping, emotion bucketing 5 levels, Laplace noise ε=1.0), `Contribution` (pseudonymous, domain-scoped), `Aggregator` (1000-user minimum threshold), `Distribution` (LoRA package building), `Bootstrap` (new user seeding from federated patterns). 8 source files, 42 tests.

### Billing (`rlmx-billing`) — ADR-025, DDD-013
**Subscription** aggregate root with 6 tiers: Free → Personal ($9.99) → Pro ($19.99) → Family ($29.99) → Developer ($49.99) → Enterprise (custom). `FamilyGroup` (max 6 members, privacy boundaries), `DeveloperAccount` (70/30 revenue split, $50 payout threshold), `UsageMetrics` (per-period tracking), `TierCapabilityEnforcer` (token-based tier enforcement with caveats). 8 source files, 55 tests.

### NAPI Binding (`rlmx-napi`) — ADR-020
**NapiKernel** DDD facade exposing full kernel syscall surface as a Node.js native addon. Methods: `dispatch` (15 syscalls), `sona_query`, `rvf_seal`, `agent_spawn`, `swarm_status`, `route`. Feature-gated behind `napi`. 29 tests.

### WASM Subset (`rlmx-wasm`) — ADR-021
Reduced kernel surface for browser/WebView (Zone D): `vec_search`, `vec_insert`, `graph_query`, `halt_check`, `state_mutate`, `sona_query`, `validate_capability`. 4-bit quantization support. Feature-gated behind `wasm`. 22 tests.

### Voice Pipeline (`rlmx-voice`) — ADR-012, DDD-008
**VoicePipeline** orchestrating VAD → STT → Intent → TTS. `VoiceActivityDetector` (energy + spectral, 3s rolling buffer), `SpeechToText` (TieredEngine escalation: Small→Medium→Remote), `MultiIntentDecomposer` (single utterance → multiple domain intents), `TextToSpeech` (6 domain personas, streaming), `VoiceSession` aggregate root with conversation turns, emotion trajectory, and session memory.

### Phone Runtime (`rlmx-phone`) — ADR-013, DDD-009
**PhoneRuntime** aggregate root with `LightweightCoordinator` (5 always-on free agents), `BackgroundScheduler` (iOS BGProcessingTask / Android WorkManager), `NotificationOrchestrator` (3-tier: Critical/Actionable/Informational with ML fatigue prevention), `BatteryAwareScheduler`, `OfflineOutbox` (bounded queue, auto-retry), `UserEngagement` (LifeScore 0-100, MoneySaved counter, Streaks with freeze support, AgentCollection levels 1-10, 50 achievements).

### Marketplace (`rlmx-marketplace`) — ADR-014, DDD-010
**Marketplace** aggregate root with `AgentRegistry` (CRUD, search, filter by domain/rating/price), `BillingEngine` (70/30 revenue split, monthly payouts at $50 threshold), `ReviewPipeline` (automated security audit + human review for sensitive permissions), `PublisherPortal`, `FeaturedEngine` (ML-ranked), `MarketplaceAnalytics`. 12 life domains. Agents packaged as RVF containers.

### MCP Server (`rlmx-mcp`)
**39 JSON-RPC 2.0 tools** (28 original + 8 marketplace + 3 voice), HTTP :3000, WebSocket :3001 with typed `SwarmEvent` enum (12 variants including VoiceChunk, AgentProgress, MultimodalResponse), auth, heartbeat, backpressure. RBAC: 6 roles, clients cannot self-escalate.

### Swarm (`rlmx-swarm`)
**6-zone topology** (A-Mobile=Phone/Primary, A-Desktop=Laptop/Secondary, B=Cloud/Burst, C=Edge/Sentinel, D=Browser, E=Mesh/Multi-Device). `SwarmEvent` (12 variants with `CardData` and `HapticPattern`). `BrowserComputePool` with priority queue. `SandboxManager` with `SandboxProfile`, `ResourceEnvelope`, `FleetManifest`. `DiscoveryService` for peer discovery.

### Agents (`rlmx-agents`)
**15 typed roles** (12 original + VoiceCoordinator, MarketplaceManager) with **15x14 permission matrix**. All agents get VoiceSynthesize + IntentRoute; only Coordinator/Router/VoiceCoordinator get VoiceTranscribe. `AgentLifecycle` state machine. Auto-research with mutation strategies.

### Cognitive (`rlmx-cognitive`)
SONA (micro-LoRA + EWC++), `VoicePatternBank` (voice-enriched patterns with temporal-weighted search), `FederatedAnonymizer` (PII stripping, emotion bucketing, Laplace noise ε=1.0), `EngagementTracker`, `NotificationFatigueModel`. `DagOptimizer`, `NervousSystem`.

**Phase 3-4 (Inference & Neural)**: `BudgetOptimizer` (LP solver for resource allocation), `LifeGraphGnn` (GNN message-passing inference), `CognitiveFramework` (meta-learning + staleness detection), `AttentionOptimizer` (flash attention + mincut).

### Tiered Inference (`rlmx-ruvllm`)
`TieredEngine` with escalation (threshold 0.4). `MlxSubprocess` for Apple Silicon. `ModelTier`: Small/ClaudeCode/Medium/Custom. **Phase 3**: Neural optimization with budget-aware model selection.

### Mobile App (`mobile/`)
React Native 0.84.1 + TypeScript for Android. **8 screens**: Home (Life Score, Money Saved, Briefing), Voice (mic button, agent progress, response cards), Agents (collection grid, levels), Insights (domain analytics), Profile (streaks, achievements), Marketplace (12 domains, featured carousel), AgentDetail, AgentCollection. **11 components** with dark theme (cyan/violet/green). Demo mode works offline. APK: `mobile/android/app/build/outputs/apk/debug/app-debug.apk`.

### Frontend (`frontend/index.html`)
**20+ views** (~2900 lines vanilla JS/CSS/HTML, no framework). Works standalone with demo data or connected to MCP server.

## Conventions

- `Arc<Mutex<T>>` or `Arc<RwLock<T>>` for shared mutable state
- Per-crate error types via `thiserror`
- `tracing` for structured logging — no `println!` in library code
- Tests are inline `#[cfg(test)]` blocks
- Feature gates: `ruvllm`, `metal`, `napi`, `wasm` are opt-in
- `SyscallPermission` defined in `rlmx-kernel`, re-exported by `rlmx-agents` — never duplicate
- Voice types (`LifeDomain`, `Intent`, `ResponseMode`, `VoicePersona`) defined in `rlmx-kernel`, used by all voice-related crates
- Mesh types (`MeshId`, `DeviceId`, `DeviceZone`) defined in `rlmx-kernel`, used by `rlmx-mesh` and `rlmx-swarm`
- Billing types (`SubscriptionTier`) defined in `rlmx-kernel`, used by `rlmx-billing` and `rlmx-marketplace`
- Aggregate roots own their consistency boundary — cross-context communication via domain events or ACL
- Mobile app uses demo data by default, switches to live when MCP server detected
- New crates must have: `lib.rs` with module declarations and re-exports, `error.rs` with `thiserror` types, `#[cfg(test)]` blocks in each source file
- CLI subcommands follow the pattern: enum variant → action enum → handler function with demo data fallback

## Strict Rules

1. **Never break the stub build**: `cargo build --workspace` without features MUST compile clean.
2. **Never add `println!` to library crates**: use `tracing::*`.
3. **Never bypass RBAC**: Admin/System only via server-side `token_roles`.
4. **Keep tests passing**: `cargo test --workspace` must pass all 925+ tests.
5. **Zero clippy warnings**: `cargo clippy --workspace -- -D warnings` must be clean.
6. **Edge tools return `"status": "unavailable"`** when no engine is configured.
7. **GGUF models require companion tokenizers**: `<model>-tokenizer.json` next to `.gguf`.
8. **MCP initialization handshake is mandatory**.
9. **Never duplicate kernel types**: import `SyscallPermission`, `Strategy`, `ProcessId`, `LifeDomain`, `Intent`, `ResponseMode`, `VoicePersona`, `MeshId`, `DeviceId`, `DeviceZone`, `SubscriptionTier`, `FederationStatus` from `rlmx-kernel`.
10. **Permission matrix is the source of truth**: const `MATRIX` in `registry.rs` per ADR-005 (15x14).
11. **Domain events must flow**: dispatch() emits SyscallDispatched, resolve_strategy() emits QueryRouted, voice sessions emit VoiceSessionStarted, mesh changes emit MeshDeviceJoined, federation cycles emit FederationCycleCompleted.
12. **Frontend fallback pattern**: Action handlers must check if server returned EMPTY data (e.g., `node_count === 0`, `agents.length === 0`), not just catch errors.
13. **Single-file frontend**: `frontend/index.html` is one file. Do NOT split into separate JS/CSS files.
14. **Voice privacy invariant**: No audio ever leaves the device. STT runs on-device. Only text transcripts and metadata flow to SONA.
15. **Federated learning privacy**: Emotion valence bucketed (5 levels), Laplace noise (ε=1.0) on urgency/satisfaction, no speaker embeddings federated, minimum 1000-user aggregation threshold. Pseudonymous contribution keys only.
16. **Phone engagement invariants**: Life Score floor at 30, savings require ProofSeal verification, streak freeze max 1 per 30 days, offline queue bounded at 100.
17. **Marketplace security**: All agents pass automated review (capability analysis, data flow verification). Sensitive permissions escalate to human review. 70/30 revenue split enforced.
18. **Mobile app must work offline**: Demo mode with realistic data when MCP server unavailable. 5 always-on agents run with zero network.
19. **Android build requires JDK 21**: Use `JAVA_HOME=/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home` — Java 26 is incompatible with React Native Gradle plugin.
20. **Mesh consistency**: PersonalMesh is the single aggregate root for device fleet management. Max 10 devices per mesh. Failover is automatic — no manual intervention required.
21. **Billing tier enforcement**: Capability tokens are derived from subscription tier, not hardcoded. Free tier limits: 5 agents, no cloud burst, no federation. Family plans max 6 members.
22. **Federation cycle integrity**: Cycles are append-only (Collecting → Aggregating → Distributing → Completed). Never skip phases. Contributions are immutable once submitted.
23. **NAPI/WASM feature isolation**: `rlmx-napi` and `rlmx-wasm` compile as normal Rust libraries without their feature flags. Feature-gated code must not leak into default builds.
24. **Format after generation**: Always run `cargo fmt` after agent-generated code. Agents do not respect rustfmt line-width limits.
25. **Capability-secured bare metal**: Phase 5 macaroon tokens use Ed25519 signatures with hierarchical narrowing. Never bypass capability validation for bare-metal operations.

## Hook Scripts

Preconfigured event hooks in `scripts/hooks/` — all executable, accept JSON stdin, output JSON stdout:

| Hook | Trigger | Purpose |
|------|---------|---------|
| `pre-voice.sh` | Before voice interaction | Check device capabilities (battery, network) |
| `post-voice.sh` | After voice interaction | Log interaction, update engagement, update streak |
| `on-savings.sh` | Savings discovered | Calculate annual impact, log to journal, notify user |
| `on-streak.sh` | Streak milestone hit | Map milestone to reward tier (3/7/14/30/60/90 days) |
| `on-agent-level.sh` | Agent levels up | Unlock capabilities (L2: batch, L7: cloud, L10: autonomous) |

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RLMX_EDGE_MODEL` | GGUF model name (without .gguf) | unset |
| `RLMX_MODEL_DIR` | Directory for GGUF + tokenizer files | `~/.rlmx/models` |
| `JAVA_HOME` | JDK path for Android builds | system default |

## Key File Locations

| Path | Purpose |
|------|---------|
| **Kernel** | |
| `crates/rlmx-kernel/src/lib.rs` | Kernel entry, module declarations |
| `crates/rlmx-kernel/src/types.rs` | SyscallPermission (16), LifeDomain, Intent, ResponseMode, VoicePersona, MeshId, DeviceId, DeviceZone, SubscriptionTier, FederationStatus |
| `crates/rlmx-kernel/src/syscall.rs` | Syscall enum (30 variants), dispatch logic |
| `crates/rlmx-kernel/src/scheduler.rs` | Strategy enum, GatherStrategy, Scheduler |
| `crates/rlmx-kernel/src/router.rs` | TinyDancerRouter (FastGRNN 18→32→5, voice-aware) |
| `crates/rlmx-kernel/src/events.rs` | DomainEvent (8+ variants), DomainEventBus |
| `crates/rlmx-kernel/src/capability.rs` | MacaroonCapabilityManager (Ed25519, Phase 5) |
| `crates/rlmx-kernel/src/proof.rs` | Phase5ProofEngine, 3-tier routing |
| `crates/rlmx-kernel/src/memory.rs` | BloomScreenedRegion, NamespacedMemoryStore (Phase 1) |
| `crates/rlmx-kernel/src/graph.rs` | EnhancedGraph, ruvector-graph dual-write (Phase 1) |
| **Voice** | |
| `crates/rlmx-voice/src/pipeline.rs` | VoicePipeline: VAD → STT → Intent → TTS |
| `crates/rlmx-voice/src/intent.rs` | MultiIntentDecomposer, IntentClassifier, 12 life domains |
| `crates/rlmx-voice/src/session.rs` | VoiceSession aggregate root, ConversationTurn, SessionMemory |
| `crates/rlmx-voice/src/vad.rs` | VoiceActivityDetector, AudioFrame, VadDecision |
| `crates/rlmx-voice/src/tts.rs` | TtsEngine, VoicePersona styles, streaming chunks |
| **Phone** | |
| `crates/rlmx-phone/src/runtime.rs` | PhoneRuntime aggregate root |
| `crates/rlmx-phone/src/coordinator.rs` | LightweightCoordinator, 5 FreeAgentType variants |
| `crates/rlmx-phone/src/notifications.rs` | NotificationOrchestrator, 3-tier priority, FatigueModel |
| `crates/rlmx-phone/src/engagement.rs` | LifeScore, MoneySaved, StreakState, AgentCollection, 50 Achievements |
| `crates/rlmx-phone/src/battery.rs` | BatteryAwareScheduler, DeviceCapabilities, BatteryPolicy |
| `crates/rlmx-phone/src/offline.rs` | OfflineOutbox, RetryPolicy |
| **Mesh** | |
| `crates/rlmx-mesh/src/mesh.rs` | PersonalMesh aggregate root, MeshId, MeshAgent, MeshDomainEvent |
| `crates/rlmx-mesh/src/device.rs` | MeshDevice, DeviceType, DeviceCapabilities, Zone |
| `crates/rlmx-mesh/src/sync.rs` | SyncProtocol, SyncState, SyncPolicy, SyncTransport |
| `crates/rlmx-mesh/src/discovery.rs` | DiscoveryService, DeviceAnnouncement, DiscoveryMethod |
| `crates/rlmx-mesh/src/failover.rs` | FailoverPolicy, DegradationLevel, MeshDegradation |
| **Federation** | |
| `crates/rlmx-federation/src/cycle.rs` | FederationCycle aggregate root (4-phase lifecycle) |
| `crates/rlmx-federation/src/anonymizer.rs` | Anonymization pipeline (PII strip, emotion bucket, Laplace noise) |
| `crates/rlmx-federation/src/contribution.rs` | Pseudonymous contributions, domain-scoped |
| `crates/rlmx-federation/src/aggregator.rs` | Cloud-side aggregation, 1000-user threshold |
| `crates/rlmx-federation/src/distribution.rs` | LoRA package building & distribution |
| `crates/rlmx-federation/src/bootstrap.rs` | New user bootstrap from federated patterns |
| **Billing** | |
| `crates/rlmx-billing/src/subscription.rs` | Subscription aggregate root, SubscriptionEvent |
| `crates/rlmx-billing/src/tier.rs` | SubscriptionTier (6 tiers), TierLimits, TierFeatures |
| `crates/rlmx-billing/src/family.rs` | FamilyGroup (max 6), FamilyRole, PrivacyBoundary |
| `crates/rlmx-billing/src/developer.rs` | DeveloperAccount, 70/30 rev share, PayoutRecord |
| `crates/rlmx-billing/src/usage.rs` | UsageMetrics, per-period tracking |
| `crates/rlmx-billing/src/capability_enforcement.rs` | TierCapabilityEnforcer, TierCapabilityToken, TierCaveat |
| **Marketplace** | |
| `crates/rlmx-marketplace/src/marketplace.rs` | Marketplace aggregate root |
| `crates/rlmx-marketplace/src/registry.rs` | AgentRegistry, AgentListing, ListingFilter |
| `crates/rlmx-marketplace/src/billing.rs` | BillingEngine, 70/30 split, payouts |
| `crates/rlmx-marketplace/src/review.rs` | ReviewPipeline, automated security checks |
| `crates/rlmx-marketplace/src/publisher.rs` | PublisherPortal, Publisher |
| `crates/rlmx-marketplace/src/featured.rs` | FeaturedEngine, ML-ranked scoring |
| **NAPI & WASM** | |
| `crates/rlmx-napi/src/lib.rs` | NapiKernel facade (dispatch, sona_query, rvf_seal, agent_spawn, swarm_status, route) |
| `crates/rlmx-wasm/src/lib.rs` | WASM kernel subset (vec_search, vec_insert, graph_query, halt_check, state_mutate, sona_query) |
| **MCP & Swarm** | |
| `crates/rlmx-mcp/src/tools.rs` | All 39 MCP tool definitions |
| `crates/rlmx-mcp/src/server.rs` | RBAC, tool dispatch, McpConfig |
| `crates/rlmx-mcp/src/ws.rs` | WebSocket server, SwarmEvent (12 variants) |
| `crates/rlmx-swarm/src/consensus.rs` | PBFT/Raft/Gossip layers |
| `crates/rlmx-swarm/src/transport.rs` | Transport layer, peer connections |
| `crates/rlmx-swarm/src/discovery.rs` | Swarm peer discovery service |
| `crates/rlmx-swarm/src/sandbox.rs` | SandboxManager, SandboxProfile, FleetManifest |
| **Agents** | |
| `crates/rlmx-agents/src/registry.rs` | PermissionMatrix (15x14), validate() |
| `crates/rlmx-agents/src/types.rs` | AgentType (15 variants incl. VoiceCoordinator, MarketplaceManager) |
| `crates/rlmx-agents/src/lifecycle.rs` | AgentLifecycle state machine |
| **Cognitive** | |
| `crates/rlmx-cognitive/src/sona.rs` | SONA micro-LoRA, PatternBank |
| `crates/rlmx-cognitive/src/voice_patterns.rs` | VoicePatternBank, FederatedAnonymizer, FatigueModel |
| `crates/rlmx-cognitive/src/gnn.rs` | LifeGraphGnn, GNN message-passing, PatternPrediction |
| `crates/rlmx-cognitive/src/solver.rs` | BudgetOptimizer, LP solver for resource allocation |
| `crates/rlmx-cognitive/src/cognitive_framework.rs` | CognitiveFramework (meta-learning, staleness detection) |
| `crates/rlmx-cognitive/src/attention.rs` | AttentionOptimizer (flash attention, mincut) |
| **Inference** | |
| `crates/rlmx-ruvllm/src/tiered.rs` | TieredEngine, escalation logic |
| `crates/rlmx-ruvllm/src/mlx_bridge.rs` | MlxSubprocess for Apple Silicon |
| `crates/rlmx-ruvllm/src/phase3.rs` | Neural optimization, budget-aware model selection |
| **Mobile** | |
| `mobile/src/screens/HomeScreen.tsx` | Life Score, Money Saved, Morning Briefing |
| `mobile/src/screens/VoiceScreen.tsx` | Mic button, transcript, agent progress cards |
| `mobile/src/screens/AgentsScreen.tsx` | Agent collection grid with levels |
| `mobile/src/screens/MarketplaceScreen.tsx` | 12 domain categories, featured carousel, install |
| `mobile/src/screens/InsightsScreen.tsx` | Domain analytics, sparklines |
| `mobile/src/screens/ProfileScreen.tsx` | Streaks, achievements, settings |
| `mobile/src/services/voice.ts` | VoiceService, demo simulation |
| `mobile/src/services/websocket.ts` | WebSocket client for SwarmEvents |
| `mobile/src/hooks/useVoice.ts` | Voice interaction React hook |
| `mobile/src/hooks/useEngagement.ts` | Life Score, savings, streaks hook |
| `mobile/src/data/agents.ts` | 52 agents (25 marketplace, 7 unlocked, 45 locked) |
| **Other** | |
| `crates/rlmx-cli/src/main.rs` | CLI: serve, swarm, agent, research, sandbox, edge, voice, marketplace, engagement, phone, mesh, billing, federation |
| `frontend/index.html` | Web dashboard (20+ views, demo data, force graph) |
| `scripts/hooks/` | 5 preconfigured event hooks |
| `docs/ADR/` | 25 Architecture Decision Records |
| `docs/DDD/` | 13 Domain-Driven Design documents |

## Concurrency: 1 MESSAGE = ALL RELATED OPERATIONS

- All operations MUST be concurrent/parallel in a single message
- ALWAYS batch ALL file reads/writes/edits in ONE message
- ALWAYS batch ALL Bash commands in ONE message
- ALWAYS use `run_in_background: true` for all agent Task calls
- ALWAYS put ALL agent Task calls in ONE message for parallel execution
- After spawning agents, STOP — wait for results

## Swarm Configuration

- Use hierarchical topology for coding swarms
- Keep maxAgents at 6-12 for tight coordination
- Use specialized strategy for clear role boundaries
- Use `raft` consensus for hive-mind
- Phone is Zone A-Mobile (primary), Laptop is Zone A-Desktop (secondary)
- Mesh zone (E) coordinates multi-device sync via PersonalMesh aggregate

## Security Rules

- NEVER hardcode API keys, secrets, or credentials in source files
- NEVER commit .env files or any file containing secrets
- Always validate user input at system boundaries
- Always sanitize file paths to prevent directory traversal
- Voice audio NEVER leaves device — privacy by design
- Federated patterns use differential privacy (Laplace ε=1.0)
- Marketplace agents must pass automated security review before publishing
- Capability tokens use Ed25519 signatures — never bypass verification
- Subscription tier limits enforced via `TierCapabilityEnforcer` — never hardcode tier checks
- Federation contributions are pseudonymous — never link to user identity

## Mobile Development

When modifying `mobile/`:
- Use TypeScript for all source files
- Use functional components with React hooks
- Dark theme with design tokens: background (#0a0e1a), surface (#141824), cyan (#00e5ff), violet (#a855f7), green (#22c55e), amber (#f59e0b), red (#ef4444), pink (#ec4899)
- All screens must work in demo mode (no server) AND live mode
- Android builds require JDK 21 — set JAVA_HOME accordingly
- APK output: `mobile/android/app/build/outputs/apk/debug/app-debug.apk`
- Use React Navigation for routing (bottom tabs + stack navigator)

## Dashboard Development

When modifying `frontend/index.html`:
- Keep file under 3000 lines total
- Use the existing design token CSS variables (--cyan, --violet, --green, --amber, --red, --pink)
- All new views go into the `viewRenderers` object as `'section.view': { desc, render, onShow }`
- New action handlers must use the fallback pattern: try server → check if empty → use DemoData
- Canvas animations must store their `requestAnimationFrame` ID and cancel on view switch
- The `SimEngine.start()` runs at 2s intervals — do not add additional setIntervals for metrics
- Test both modes: with MCP server running (`:3000`) and without (demo-only)

**WASM init**: call `await mod.default()` before any other API. Do NOT call `mod.init()` after.

## Distributed Deployment

| Target | Method | Status |
|--------|--------|--------|
| Android phone | React Native APK | Working |
| Local Mac (multi-tab) | BroadcastChannel | Working |
| RPi5 / NUC (bare metal) | Cross-compile + systemd | Deploy config in `deploy/` |
| Cloud GPU burst | SkyPilot | Planned (ADR-006) |
| Browser workers | WASM compute pool | Working (ADR-009) |
| Node.js servers | NAPI native addon | Working (ADR-020) |
| Multi-device mesh | PersonalMesh sync | Working (ADR-022) |

## DDD Bounded Contexts (13)

| # | Context | Crate | Aggregate Root |
|---|---------|-------|----------------|
| 1 | Kernel Syscall | `rlmx-kernel` | KernelContext |
| 2 | Agent Lifecycle | `rlmx-agents` | AgentRegistry |
| 3 | Swarm Coordination | `rlmx-swarm` | SwarmCluster |
| 4 | Inference Routing | `rlmx-kernel`, `rlmx-ruvllm` | Scheduler |
| 5 | Research & Evolution | `rlmx-agents` | Researcher |
| 6 | Observation & Health | `rlmx-cognitive` | Sona |
| 7 | Container & Storage | `rlmx-rvf` | RvfContainer |
| 8 | Voice Interaction | `rlmx-voice` | VoiceSession |
| 9 | Phone Runtime | `rlmx-phone` | PhoneRuntime |
| 10 | Agent Marketplace | `rlmx-marketplace` | Marketplace |
| 11 | Personal Mesh | `rlmx-mesh` | PersonalMesh |
| 12 | Federated Learning | `rlmx-federation` | FederationCycle |
| 13 | Subscription Billing | `rlmx-billing` | Subscription |

## ADR Index

| ADR | Title | Status |
|-----|-------|--------|
| 001 | Distributed Swarm Architecture | Accepted |
| 002 | Layered Consensus Protocol | Accepted |
| 003 | Neural Model Routing | Accepted |
| 004 | Tiered Model Inference | Accepted |
| 005 | Capability-Secured Agents | Accepted |
| 006 | Evolutionary Auto-Research | Accepted |
| 007 | RuVix Kernel Migration | Accepted |
| 008 | WebSocket Realtime Events | Accepted |
| 009 | Browser WASM Compute Pool | Accepted |
| 010 | MCP Tool Expansion | Accepted |
| 011 | Sandbox Orchestration | Accepted |
| 012 | Voice-First Pipeline | Implemented |
| 013 | Phone Command Center | Implemented |
| 014 | Agent Marketplace | Implemented |
| 015 | Multi-Intent Fan-Out | Implemented |
| 016 | Engagement & Gamification | Implemented |
| 017 | Federated Voice Learning | Implemented |
| 018 | Multimodal Response Protocol | Implemented |
| 019 | Kernel Expansion | Implemented |
| 020 | NAPI-RS Native Binding | Implemented |
| 021 | WASM Kernel Subset | Implemented |
| 022 | Personal Mesh Topology | Implemented |
| 023 | Federated Learning Pipeline | Implemented |
| 024 | Ruvnet Crate Integration (Phase 1-5) | Implemented |
| 025 | Subscription Billing Tiers | Implemented |

## Support

- Documentation: https://github.com/ruvnet/claude-flow
- Issues: https://github.com/ruvnet/claude-flow/issues
