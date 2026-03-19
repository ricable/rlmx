# Architecture — Detailed Component Reference

## Interaction Paths

1. **Voice-First**: On-device STT (Whisper-tiny Q4) -> TinyDancerRouter (18-dim) -> Multi-Intent Decomposition -> Agent Swarm -> Multimodal Response (voice + cards + haptics)
2. **Cloud/Dev**: `VllmClient` -> vLLM on Metal (Mac) or NVIDIA GPU (prod)
3. **Edge**: `LocalEngine` -> ruvllm CandleBackend on CPU/Metal/CUDA with GGUF models
4. **Tiered**: `TieredEngine` -> Small (0.5B) -> Medium (MLX 3-8B) -> Remote (vLLM) with confidence escalation
5. **Browser**: `@ruvector/ruvllm-wasm` -> WebGPU/WASM SIMD (inference primitives + HTTP fallback)
6. **Swarm**: Cross-zone scatter-gather via `Strategy::Swarm` with configurable gather strategy

## Kernel (`rlmx-kernel`)

**18-syscall dispatch** (12 original + VoiceTranscribe, VoiceSynthesize, IntentRoute, MeshSync, FederationContribute, ArtifactWrite) with `DomainEventBus` (21 cross-context events including VoiceSessionStarted, IntentsDecomposed, MeshDeviceJoined, FederationCycleCompleted, ArtifactCreated, BoardPostCreated, BudgetThresholdReached, FunctionEvolved, ApprovalRequested, ChannelMessageReceived). New kernel modules: `approval.rs` (ApprovalTier: Auto/Notify/Confirm/Escalate with 14 default policies), `trigger.rs` (TriggerType: Http/Schedule/Event/Channel with cycle prevention), `a2a.rs` (AgentCard builder for all 17 agent types). `TinyDancerRouter` (FastGRNN **18->32->5**) with 4 voice-aware dimensions (speaker_confidence, emotion_valence, urgency_score, ambient_noise_level) and online learning with 0.6 confidence gating. `Strategy` enum includes `Swarm { scatter_zones, gather_strategy, timeout_ms }`.

Types: `ResponseMode` (VoiceOnly/Visual/Multimodal/Ambient), `VoicePersona` (6 domain personas), `LifeDomain` (12 life categories), `Intent` struct.

## Voice Pipeline (`rlmx-voice`) — ADR-012, DDD-008

**VoicePipeline** orchestrating VAD -> STT -> Intent -> TTS. `VoiceActivityDetector` (energy + spectral, 3s rolling buffer), `SpeechToText` (TieredEngine escalation: Small->Medium->Remote), `MultiIntentDecomposer` (single utterance -> multiple domain intents), `TextToSpeech` (6 domain personas, streaming), `VoiceSession` aggregate root with conversation turns, emotion trajectory, and session memory.

## Phone Runtime (`rlmx-phone`) — ADR-013, DDD-009

**PhoneRuntime** aggregate root with `LightweightCoordinator` (5 always-on free agents), `BackgroundScheduler` (iOS BGProcessingTask / Android WorkManager), `NotificationOrchestrator` (3-tier: Critical/Actionable/Informational with ML fatigue prevention), `BatteryAwareScheduler`, `OfflineOutbox` (bounded queue, auto-retry), `UserEngagement` (LifeScore 0-100, MoneySaved counter, Streaks with freeze support, AgentCollection levels 1-10, 50 achievements).

## Marketplace (`rlmx-marketplace`) — ADR-014, DDD-010

**Marketplace** aggregate root with `AgentRegistry` (CRUD, search, filter by domain/rating/price), `BillingEngine` (70/30 revenue split, monthly payouts at $50 threshold), `ReviewPipeline` (automated security audit + human review for sensitive permissions), `PublisherPortal`, `FeaturedEngine` (ML-ranked), `MarketplaceAnalytics`. 12 life domains. Agents packaged as RVF containers.

## MCP Server (`rlmx-mcp`)

**82 JSON-RPC 2.0 tools** (28 original + 8 marketplace + 3 voice + 2 mesh + 2 federation + 4 billing + 35 AgentOS integration), HTTP :3000, WebSocket :3001 with typed `SwarmEvent` enum (14 variants including VoiceChunk, AgentProgress, MultimodalResponse, BoardUpdate, ApprovalRequired), auth, heartbeat, backpressure. RBAC: 6 roles, clients cannot self-escalate.

## Swarm (`rlmx-swarm`)

**6-zone topology** (A-Mobile=Phone/Primary, A-Desktop=Laptop/Secondary, B=Cloud/Burst, C=Edge/Sentinel, D=Browser, E=HomeHub/PrivacyAnchor). `SwarmEvent` expanded to 14 variants with `CardData`, `HapticPattern`, `BoardUpdate`, and `ApprovalRequired` types. `BrowserComputePool` with priority queue. `SandboxManager` with `SandboxProfile`, `ResourceEnvelope`, `FleetManifest`. **Coordination Board** (`board.rs`): persistent inter-agent communication with threaded posts, pinning (25/board), 1,000 post limit, artifact attachments.

## Agents (`rlmx-agents`)

**17 typed roles** (12 original + VoiceCoordinator, MarketplaceManager, MeshCoordinator, FederationAgent, BillingManager) with **18x17 permission matrix** (ArtifactWrite added in ADR-030; granted to Coordinator, Researcher, Experimenter). All agents get VoiceSynthesize + IntentRoute; only Coordinator/Router/VoiceCoordinator get VoiceTranscribe. MeshCoordinator gets MeshSync; FederationAgent gets FederationContribute. `AgentLifecycle` state machine. Auto-research with mutation strategies.

## Cognitive (`rlmx-cognitive`)

SONA (micro-LoRA + EWC++), `VoicePatternBank` (voice-enriched patterns with temporal-weighted search), `FederatedAnonymizer` (PII stripping, emotion bucketing, Laplace noise e=1.0), `EngagementTracker`, `NotificationFatigueModel`. DagOptimizer, NervousSystem.

## Tiered Inference (`rlmx-ruvllm`)

`TieredEngine` with escalation (threshold 0.4). `MlxSubprocess` for Apple Silicon. `ModelTier`: Small/ClaudeCode/Medium/Custom.

## NAPI Bindings (`rlmx-napi`) — ADR-020

**NapiKernel** aggregate root exposing kernel operations via napi-rs: `dispatch`, `sona_query`, `rvf_seal`, `agent_spawn`, `swarm_status`, `route`. Sub-millisecond syscall dispatch from Node.js without HTTP overhead. Published as `@ruvix/mesh-native`.

## WASM Module (`rlmx-wasm`) — ADR-021

Browser-side kernel syscall subset via `wasm-bindgen`: VecSearch, VecInsert, GraphQuery, HaltCheck, StateMutate, SONA query. 4-bit quantized vectors (50K in ~25MB). IndexedDB backing for persistence.

## Personal Mesh (`rlmx-mesh`) — ADR-022, DDD-011

**PersonalMesh** aggregate root coordinating cross-device personal mesh. mDNS discovery on LAN, WebSocket relay for remote. Privacy anchor pattern: home hub (Zone C/E) is sole long-term data store. QUIC transport for sub-millisecond LAN sync. Own domain events via `MeshDomainEvent`.

## Federated Learning (`rlmx-federation`) — ADR-023, DDD-012

**FederationCycle** aggregate root managing weekly federated learning cycle. On-device anonymization via `FederationAnonymizer`, cloud aggregation with 1000-user minimum threshold, bidirectional distribution via RVF containers. Per-user continuous learning with micro-LoRA + EWC++.

## Subscription Billing (`rlmx-billing`) — ADR-025, DDD-013

**Subscription** aggregate root with 6-tier model (Free/Personal/Family/Pro/Enterprise/Developer). Capability-token-enforced limits via Macaroon caveats (including `BudgetLimit`). Free tier: exactly 5 agents. Cloud burst metering by inference tokens. Stripe integration. 7-day grace period. 70/30 developer/platform revenue split. **Per-Agent Budget Ledger** (`budget.rs`, ADR-032): BudgetLedger with CAS versioning, BudgetPolicy (soft/hard limits), per-agent cost breakdown by model/provider.

## Mobile App (`mobile/`)

React Native 0.84.1 + TypeScript for Android. **8 screens**: Home, Voice, Agents, Insights, Profile, Marketplace, AgentDetail, AgentCollection. **11 components** with dark theme (cyan/violet/green). Demo mode works offline. APK: `mobile/android/app/build/outputs/apk/debug/app-debug.apk`.

## Artifact DAG (`rlmx-artifact`) — ADR-030

**ContentAddressedStore** aggregate root providing SHA-256 content-addressed artifact versioning. `ArtifactDag` for parent validation and lineage traversal. `BranchManager` for named branch heads. Agents produce versioned, diffable outputs consumed by other agents or swarm workflows.

## Dynamic Function Evolution (`rlmx-evolve`) — ADR-036

**LifecycleManager** orchestrating function evolution: draft→staging→production→deprecated→killed. `ScoreEngine` with formula (correctness×0.50 + safety×0.25 + latency×0.15 + cost×0.10). `FeedbackEngine` auto-reviews last 5 evals (KILL ≥3 failures, IMPROVE avg<0.5, KEEP otherwise). `FunctionDag` for version graph with fork/lineage/leaves. Adapted from AgentOS pattern; uses RLMX WASM sandbox (stronger than AgentOS node:vm).

## Channel Adapters (`rlmx-channels`) — ADR-039

**ChannelRegistry** managing platform adapters via `ChannelAdapter` trait (async send/receive/health_check). Stub implementations for Telegram (Bot API v7), WhatsApp (Cloud API v19), Teams (Bot Framework v4), Discord (Gateway v10). Integrates with Phase 9 triggers and Phase 8 approval routing.

## A2A Protocol (`@aix/a2a`) — ADR-034

**A2AServer** implementing JSON-RPC 2.0 A2A protocol. `AgentCard` exposes all 17 agent types as discoverable skills at `/.well-known/agent.json`. `TaskStore` with FIFO eviction (max 1,000). Task state machine: submitted→working→input-required→completed|cancelled|failed.

## Skills System (`@aix/skills`) — ADR-035

**SkillRegistry** with 12 bundled skills (one per LifeDomain). SKILL.md parser extracts YAML frontmatter metadata. Security validation pipeline. Semantic search across installed skills.

## Trigger Registry (`rlmx-kernel/trigger.rs`) — ADR-038

**TriggerRegistry** binding external events to MCP tool functions. Four trigger types: Http (webhooks), Schedule (cron), Event (DomainEvent pattern match), Channel (messaging platform messages). Cycle prevention via max depth 5. Integrates with all other AgentOS-derived subsystems.

## Frontend (`frontend/index.html`)

**20+ views** (~2900 lines vanilla JS/CSS/HTML, no framework). Works standalone with demo data or connected to MCP server.
