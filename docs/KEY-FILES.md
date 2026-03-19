# Key File Locations

## Kernel

| Path | Purpose |
|------|---------|
| `crates/rlmx-kernel/src/lib.rs` | Kernel types: SyscallPermission (18), LifeDomain, Intent, ResponseMode, VoicePersona |
| `crates/rlmx-kernel/src/scheduler.rs` | Strategy enum, GatherStrategy, Scheduler |
| `crates/rlmx-kernel/src/router.rs` | TinyDancerRouter (FastGRNN 18->32->5, voice-aware) |
| `crates/rlmx-kernel/src/events.rs` | DomainEvent (21 variants), DomainEventBus |
| `crates/rlmx-kernel/src/approval.rs` | ApprovalTier, ApprovalGate, ApprovalPolicy (14 default policies, ADR-037) |
| `crates/rlmx-kernel/src/trigger.rs` | TriggerType, TriggerBinding, TriggerRegistry (cycle prevention, ADR-038) |
| `crates/rlmx-kernel/src/a2a.rs` | AgentCard, AgentCardBuilder, A2ASkill, TaskState (ADR-034) |

## Voice

| Path | Purpose |
|------|---------|
| `crates/rlmx-voice/src/pipeline.rs` | VoicePipeline: VAD -> STT -> Intent -> TTS |
| `crates/rlmx-voice/src/intent.rs` | MultiIntentDecomposer, IntentClassifier, 12 life domains |
| `crates/rlmx-voice/src/session.rs` | VoiceSession aggregate root, ConversationTurn, SessionMemory |
| `crates/rlmx-voice/src/vad.rs` | VoiceActivityDetector, AudioFrame, VadDecision |
| `crates/rlmx-voice/src/tts.rs` | TtsEngine, PersonaConfig, PersonaStyle, streaming chunks |

## Phone

| Path | Purpose |
|------|---------|
| `crates/rlmx-phone/src/runtime.rs` | PhoneRuntime aggregate root |
| `crates/rlmx-phone/src/coordinator.rs` | LightweightCoordinator, 5 FreeAgentType variants |
| `crates/rlmx-phone/src/notifications.rs` | NotificationOrchestrator, 3-tier priority, FatigueModel |
| `crates/rlmx-phone/src/engagement.rs` | LifeScore, MoneySaved, StreakState, AgentCollection, 50 Achievements |
| `crates/rlmx-phone/src/battery.rs` | BatteryAwareScheduler, DeviceCapabilities, BatteryPolicy |
| `crates/rlmx-phone/src/offline.rs` | OfflineOutbox, RetryPolicy |

## Marketplace

| Path | Purpose |
|------|---------|
| `crates/rlmx-marketplace/src/marketplace.rs` | Marketplace aggregate root |
| `crates/rlmx-marketplace/src/registry.rs` | AgentRegistry, AgentListing, ListingFilter |
| `crates/rlmx-marketplace/src/billing.rs` | BillingEngine, 70/30 split, payouts |
| `crates/rlmx-marketplace/src/review.rs` | ReviewPipeline, automated security checks |
| `crates/rlmx-marketplace/src/publisher.rs` | PublisherPortal, Publisher |
| `crates/rlmx-marketplace/src/featured.rs` | FeaturedEngine, ML-ranked scoring |

## MCP & Swarm

| Path | Purpose |
|------|---------|
| `crates/rlmx-mcp/src/tools.rs` | All 47 MCP tool definitions |
| `crates/rlmx-mcp/src/server.rs` | RBAC, tool dispatch, McpConfig |
| `crates/rlmx-mcp/src/ws.rs` | WebSocket server, SwarmEvent dispatch (14 variants incl. BoardUpdate, ApprovalRequired) |
| `crates/rlmx-swarm/src/board.rs` | BoardManager, Board, Post, threading/pinning (ADR-031) |
| `crates/rlmx-swarm/src/consensus.rs` | PBFT/Raft/Gossip layers |
| `crates/rlmx-swarm/src/sandbox.rs` | SandboxManager, SandboxProfile, FleetManifest |

## Agents

| Path | Purpose |
|------|---------|
| `crates/rlmx-agents/src/registry.rs` | PermissionRegistry (18x17), validate() |
| `crates/rlmx-agents/src/types.rs` | AgentType (17 variants) |
| `crates/rlmx-agents/src/lifecycle.rs` | AgentLifecycle state machine |

## Cognitive

| Path | Purpose |
|------|---------|
| `crates/rlmx-cognitive/src/sona.rs` | SONA micro-LoRA, PatternBank |
| `crates/rlmx-cognitive/src/voice_patterns.rs` | VoicePatternBank, FederatedAnonymizer, FatigueModel |

## NAPI & WASM

| Path | Purpose |
|------|---------|
| `crates/rlmx-napi/src/lib.rs` | NapiKernel aggregate root, NAPI functions |
| `crates/rlmx-wasm/src/lib.rs` | WASM kernel subset |

## Mesh

| Path | Purpose |
|------|---------|
| `crates/rlmx-mesh/src/mesh.rs` | PersonalMesh aggregate root, MeshDomainEvent |
| `crates/rlmx-mesh/src/device.rs` | MeshDevice, DeviceId, DeviceType, Zone |
| `crates/rlmx-mesh/src/discovery.rs` | DiscoveryService, mDNS + WebSocket relay |
| `crates/rlmx-mesh/src/sync.rs` | SyncState, SyncPolicy, SyncProtocol |
| `crates/rlmx-mesh/src/failover.rs` | FailoverPolicy, DegradationLevel |

## Federation

| Path | Purpose |
|------|---------|
| `crates/rlmx-federation/src/cycle.rs` | FederationCycle aggregate root, CycleStatus |
| `crates/rlmx-federation/src/anonymizer.rs` | FederationAnonymizer, on-device PII stripping |
| `crates/rlmx-federation/src/contribution.rs` | Contribution, pseudonymous keys |
| `crates/rlmx-federation/src/aggregator.rs` | FederatedAggregator, 1000-user threshold |
| `crates/rlmx-federation/src/distribution.rs` | FederationPackage, PackageDistributor |
| `crates/rlmx-federation/src/bootstrap.rs` | New user bootstrap from federated patterns |

## Billing

| Path | Purpose |
|------|---------|
| `crates/rlmx-billing/src/subscription.rs` | Subscription aggregate, SubscriptionEvent |
| `crates/rlmx-billing/src/tier.rs` | SubscriptionTier (6 tiers), TierLimits, TierFeatures |
| `crates/rlmx-billing/src/capability_enforcement.rs` | TierCapabilityEnforcer, Macaroon caveats |
| `crates/rlmx-billing/src/family.rs` | FamilyGroup, FamilyMember, 6-member max |
| `crates/rlmx-billing/src/developer.rs` | DeveloperAccount, 70/30 revenue share |
| `crates/rlmx-billing/src/usage.rs` | UsageMetrics, cloud token metering |
| `crates/rlmx-billing/src/budget.rs` | BudgetLedger, BudgetPolicy, BudgetDecision, CAS versioning (ADR-032) |

## Mobile

| Path | Purpose |
|------|---------|
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

## TypeScript Packages

| Path | Purpose |
|------|---------|
| `packages/shared/src/enums.ts` | SyscallPermission(18), LifeDomain(12), AgentType(17), SubscriptionTier(6) |
| `packages/shared/src/events.ts` | DomainEvent discriminated union (21 variants), DomainEventBus |
| `packages/shared/src/errors.ts` | AixError, AixErrorCode (JSON-RPC + kernel + agent + billing ranges) |
| `packages/core/src/loader.ts` | Platform detection (5 triples), NAPI binary loading, degraded proxy |
| `packages/core/src/types.ts` | All 31 NAPI function type declarations |
| `packages/agents/src/registry.ts` | PermissionRegistry (18x17 matrix as Map), validate() |
| `packages/agents/src/lifecycle.ts` | AgentLifecycle state machine (6 states) |
| `packages/mcp-server/src/tools.ts` | All 47 MCP tool handlers |
| `packages/mcp-server/src/rbac.ts` | 6-role RBAC, toolToOperation mapping, anti-escalation |
| `packages/marketplace/src/marketplace.ts` | Marketplace aggregate root |
| `packages/mesh/src/mesh.ts` | PersonalMesh aggregate root |
| `packages/billing/src/subscription.ts` | Subscription aggregate with state machine |
| `packages/federation/src/cycle.ts` | FederationCycle aggregate root |
| `packages/swarm/src/consensus.ts` | PBFT/Raft/Gossip consensus layers |
| `packages/aix/src/index.ts` | CLI entry: createProgram(), 14 command groups |

## Deploy

| Path | Purpose |
|------|---------|
| `packages/deploy/src/manifest.ts` | AgentManifest, AgentOrigin (4 variants), TransportSpec (7 variants) |
| `packages/deploy/src/templates.ts` | 50 frozen templates, O(1) lookup by id/domain/origin |
| `packages/deploy/src/registry.ts` | ManifestRegistry (CRUD + single-pass AND search) |
| `packages/deploy/src/bridge.ts` | DiscoveryBridge — unified mDNS (_cognitum._tcp + _rlmx._tcp) |
| `packages/deploy/src/seed-bridge.ts` | SeedBridge — Cognitum Seed REST/MCP client (EMBED_DIM=64) |

## Artifact DAG

| Path | Purpose |
|------|---------|
| `crates/rlmx-artifact/src/types.rs` | ArtifactId (SHA-256 newtype), Artifact, ArtifactDiff, ArtifactError |
| `crates/rlmx-artifact/src/dag.rs` | ArtifactDag: insert, get, parents, children, lineage, roots |
| `crates/rlmx-artifact/src/store.rs` | ContentAddressedStore: SHA-256 hashing, diff |
| `crates/rlmx-artifact/src/branch.rs` | BranchManager: named branch heads, advance, merge |

## Function Evolution

| Path | Purpose |
|------|---------|
| `crates/rlmx-evolve/src/types.rs` | FunctionId, EvolvedFunction, FunctionStatus, ScoreResult, FeedbackDecision |
| `crates/rlmx-evolve/src/lifecycle.rs` | LifecycleManager: draft→staging→production→deprecated→killed |
| `crates/rlmx-evolve/src/scorer.rs` | ScoreEngine: exact_match, semantic_similarity, compute_overall |
| `crates/rlmx-evolve/src/feedback.rs` | FeedbackEngine: KILL/IMPROVE/KEEP decisions, leaderboard |
| `crates/rlmx-evolve/src/dag.rs` | FunctionDag: fork, lineage, leaves |

## Channel Adapters

| Path | Purpose |
|------|---------|
| `crates/rlmx-channels/src/types.rs` | ChannelId, ChannelMessage, ChannelConfig, ChannelError |
| `crates/rlmx-channels/src/registry.rs` | ChannelAdapter trait, ChannelRegistry |
| `crates/rlmx-channels/src/telegram.rs` | TelegramAdapter (Bot API v7) |
| `crates/rlmx-channels/src/whatsapp.rs` | WhatsAppAdapter (Cloud API v19) |
| `crates/rlmx-channels/src/teams.rs` | TeamsAdapter (Bot Framework v4) |
| `crates/rlmx-channels/src/discord.rs` | DiscordAdapter (Gateway v10) |

## AgentOS Integration TS Packages

| Path | Purpose |
|------|---------|
| `packages/artifact/src/index.ts` | ArtifactStore, BoardManager (in-memory DAG client) |
| `packages/a2a/src/types.ts` | AgentCard, A2ATask, TaskState, Part types |
| `packages/a2a/src/server.ts` | A2AServer: tasks/send, tasks/get, tasks/cancel JSON-RPC |
| `packages/a2a/src/discovery.ts` | buildAgentCard() — all 17 agent types as A2A skills |
| `packages/a2a/src/task-store.ts` | TaskStore with FIFO eviction (max 1,000) |
| `packages/skills/src/registry.ts` | SkillRegistry: list, get, install, search |
| `packages/skills/src/parser.ts` | parseSkillMd() — YAML frontmatter + body |
| `packages/skills/src/bundled.ts` | 12 bundled skills (one per LifeDomain) |
| `packages/evolve/src/lifecycle.ts` | LifecycleManager: create, transition, list |
| `packages/evolve/src/scorer.ts` | ScoreEngine: exactMatch, semanticSimilarity, computeOverall |
| `packages/evolve/src/feedback.ts` | FeedbackEngine: review (KILL/IMPROVE/KEEP), leaderboard |
| `packages/triggers/src/registry.ts` | TriggerRegistry: bind, unbind, match event/HTTP/channel |
| `packages/channels/src/registry.ts` | ChannelRegistry: register, send, list, health |
| `packages/deploy/src/bridge-claude-code.ts` | ClaudeCodeBridgeAdapter (CLI subprocess) |
| `packages/deploy/src/bridge-codex.ts` | CodexBridgeAdapter (HTTP) |
| `packages/deploy/src/bridge-http-generic.ts` | HttpGenericBridgeAdapter (OpenAI-compatible) |

## Other

| Path | Purpose |
|------|---------|
| `crates/rlmx-cli/src/main.rs` | CLI: 14 command groups |
| `frontend/index.html` | Web dashboard (20+ views, demo data) |
| `scripts/hooks/` | 5 preconfigured event hooks |
| `docs/ADR/` | 39 Architecture Decision Records (ADR-001 through ADR-039) |
| `docs/DDD/` | 16 Domain-Driven Design documents |
