 Plan: Package RLMX as npx aix (ruvector pattern)                                                          
                                                                                                        
 Context                                     

 RLMX has 19 Rust crates (~49K LOC). Analysis shows 57% is orchestration/business logic better suited to
 TypeScript, while 34% is compute-heavy code (neural nets, vectors, crypto) that must stay Rust. The user
 wants to package this as npx aix following the exact ruvector packaging model: NAPI per-platform
 binaries, standalone WASM, TypeScript CLI with commander/chalk/ora. Namespace is @aix. Voice and phone
 crates stay in Rust.

 Package Map

 RUST (via @aix/core NAPI + @aix/wasm)     TYPESCRIPT (npm packages)
 ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━       ━━━━━━━━━━━━━━━━━━━━━━━━
 rlmx-kernel   (router, vectors, proofs)    rlmx-cli     → aix (CLI)
 rlmx-cognitive (SONA, LoRA, GNN)           rlmx-mcp     → @aix/mcp-server
 rlmx-ruvllm   (Candle, Metal/CUDA)         rlmx-agents  → @aix/agents
 rlmx-trm      (recursive NN)              rlmx-marketplace → @aix/marketplace
 rlmx-rvf      (Ed25519, witness)           rlmx-mesh    → @aix/mesh
 rlmx-voice    (VAD, STT, TTS)             rlmx-billing → @aix/billing
 rlmx-phone    (runtime, engagement)        rlmx-swarm   → @aix/swarm
 rlmx-wasm     (browser kernel)             rlmx-federation → @aix/federation
                                            rlmx-plugin  → @aix/plugin
                                            rlmx-rlm     → @aix/rlm

 Key Decisions

 - Scope: All 5 phases in one pass — deliver complete npx aix experience
 - NAPI: Single @aix/core binary bundling kernel+cognitive+voice+phone+rvf+trm+ruvllm (~15-20MB per
 platform)
 - Archive: Ported Rust crates move to crates/archive/ as their TS replacements are built
 - Tests: Port Rust #[cfg(test)] blocks to vitest equivalents for each TS package
 - Build: tsup (esbuild-based) for CJS + ESM + .d.ts generation
 - MCP: Use official @modelcontextprotocol/sdk (not custom JSON-RPC port)

 Directory Structure

 rlmx/
   packages/
     aix/                        # `npx aix` — CLI entry (commander+chalk+ora)
       bin/cli.js
       src/commands/              # serve, query, voice, agent, mesh, billing...
       package.json               # depends on all @aix/* packages
     core/                        # @aix/core — NAPI JS loader (~11KB)
       index.js                   # platform detection + dynamic require
       index.d.ts                 # generated TypeScript declarations
       package.json               # optionalDeps on per-platform binaries
     core-darwin-arm64/           # @aix/core-darwin-arm64 — .node binary
     core-darwin-x64/             # @aix/core-darwin-x64
     core-linux-x64-gnu/          # @aix/core-linux-x64-gnu
     core-linux-arm64-gnu/        # @aix/core-linux-arm64-gnu
     core-win32-x64-msvc/         # @aix/core-win32-x64-msvc
     wasm/                        # @aix/wasm — standalone WASM (wasm-pack output)
       aix_wasm.js / .d.ts / _bg.wasm
     shared/                      # @aix/shared — shared TS types (zero deps)
     mcp-server/                  # @aix/mcp-server — JSON-RPC/HTTP/WS
     agents/                      # @aix/agents — permissions, lifecycle
     marketplace/                 # @aix/marketplace — CRUD, reviews, billing
     mesh/                        # @aix/mesh — device discovery, sync
     billing/                     # @aix/billing — Stripe, tiers
     swarm/                       # @aix/swarm — orchestration
     federation/                  # @aix/federation — cycle management
     plugin/                      # @aix/plugin — plugin registry
     rlm/                         # @aix/rlm — vLLM HTTP client
   crates/
     rlmx-kernel/                 # STAYS — compiled into @aix/core
     rlmx-cognitive/              # STAYS — compiled into @aix/core
     rlmx-ruvllm/                 # STAYS — compiled into @aix/core
     rlmx-trm/                    # STAYS — compiled into @aix/core
     rlmx-rvf/                    # STAYS — compiled into @aix/core
     rlmx-voice/                  # STAYS — compiled into @aix/core
     rlmx-phone/                  # STAYS — compiled into @aix/core
     rlmx-napi/                   # STAYS — the NAPI bridge itself
     rlmx-wasm/                   # STAYS — compiled to @aix/wasm
     archive/                     # ARCHIVED — replaced by TypeScript packages
       rlmx-cli/                  # → aix CLI
       rlmx-mcp/                  # → @aix/mcp-server
       rlmx-agents/               # → @aix/agents
       rlmx-swarm/                # → @aix/swarm
       rlmx-marketplace/          # → @aix/marketplace
       rlmx-mesh/                 # → @aix/mesh
       rlmx-billing/              # → @aix/billing
       rlmx-federation/           # → @aix/federation
       rlmx-plugin/               # → @aix/plugin
       rlmx-rlm/                  # → @aix/rlm
   package.json                   # Root: { "workspaces": ["packages/*"] }

 Phase 1: Foundation (no inter-package deps)

 Step 1.1: Create root package.json + workspace config

 - File: /rlmx/package.json — { "private": true, "workspaces": ["packages/*"] }
 - File: /rlmx/tsconfig.base.json — shared TS compiler options (target ES2022, module NodeNext, strict)
 - Each package uses tsup for build: tsup src/index.ts --format cjs,esm --dts
 - Each package uses vitest for testing

 Step 1.2: Create @aix/shared (pure TS types)

 - Dir: packages/shared/
 - Port kernel types from crates/rlmx-kernel/src/lib.rs:
   - SyscallPermission (17 variants) → TypeScript enum
   - LifeDomain (12 variants) → TypeScript enum
   - Intent, ResponseMode, VoicePersona → TypeScript interfaces
   - AgentType (17 variants) → TypeScript enum
   - Strategy, GatherStrategy → TypeScript types
   - DomainEvent (10 variants) → TypeScript discriminated union
 - Port billing types from crates/rlmx-billing/src/tier.rs:
   - SubscriptionTier (6 tiers), TierLimits, TierFeatures
 - Utility: JSON-RPC helpers, error types
 - No deps on @aix/core — pure type definitions

 Step 1.3: Expand rlmx-napi + build @aix/core

 - File: crates/rlmx-napi/Cargo.toml — add deps on rlmx-voice, rlmx-phone, rlmx-cognitive, rlmx-rvf,
 rlmx-trm
 - File: crates/rlmx-napi/src/lib.rs — add ~25 new NAPI functions:

 - Voice (6 functions):
   - napi_voice_pipeline_create(), napi_voice_vad_process(energy, spectral, duration_ms)
   - napi_voice_transcribe(audio_len, language, tier), napi_voice_decompose_intents(transcript, max)
   - napi_voice_synthesize(text, persona, streaming), napi_voice_session_create/add_turn/status()

 Phone (8 functions):
   - napi_phone_runtime_create(device_id, capabilities), napi_phone_engagement_score()
   - napi_phone_savings_record(cents, domain, proof), napi_phone_streak_check_in/status()
   - napi_phone_battery_policy(level, thermal), napi_phone_coordinator_status()
   - napi_phone_notification_send(title, body, priority)

 Cognitive (5 functions):
   - napi_sona_record(query, actions, quality), napi_sona_adapt(embedding, quality)
   - napi_voice_pattern_search(query, top_k), napi_federated_anonymize(pattern_json)
   - napi_fatigue_check(user_id)

 RVF (1 function): napi_rvf_verify(witness_id)
 - NapiKernel struct grows to hold: VoicePipeline, PhoneRuntime, Sona, VoicePatternBank, HashMap<Uuid,
 VoiceSession>
 - Build: packages/core/ — napi-rs generates loader + .d.ts + per-platform .node files
 - Existing 6 functions unchanged: dispatch, sona_query, rvf_seal, agent_spawn, swarm_status, route

 Step 1.4: Build @aix/wasm

 - Build: wasm-pack build crates/rlmx-wasm --target web --out-dir ../../packages/wasm --out-name aix_wasm
 -- --features wasm
 - Outputs: aix_wasm.js, aix_wasm.d.ts, aix_wasm_bg.wasm
 - Package.json: { "name": "@aix/wasm", "type": "module", "main": "aix_wasm.js" }

 Phase 2: Leaf packages (depend on shared + core only)

 Step 2.1: @aix/rlm

 - Port crates/rlmx-rlm/src/ (1,829 LOC) → TypeScript
 - VllmClient class using fetch (replaces reqwest)
 - Exports: VllmClient, VllmConfig, ChatMessage, CompletionRequest
 - Deps: @aix/shared

 Step 2.2: @aix/plugin

 - Port crates/rlmx-plugin/src/ (1,883 LOC) → TypeScript
 - PluginRegistry with dynamic import support
 - Exports: PluginRegistry, PluginManifest, DomainPlugin interface
 - Deps: @aix/shared

 Step 2.3: @aix/billing

 - Port crates/rlmx-billing/src/ (1,707 LOC) → TypeScript
 - Subscription, TierLimits, FamilyGroup, DeveloperAccount, UsageMetrics
 - Stripe SDK integration (replaces stub)
 - Deps: @aix/core, @aix/shared, stripe

 Phase 3: Mid-level packages

 Step 3.1: @aix/agents

 - Port crates/rlmx-agents/src/ (4,733 LOC) → TypeScript
 - PermissionRegistry (17x17 matrix as TypeScript Map)
 - AgentLifecycle state machine
 - Spawning calls @aix/core.napiAgentSpawn()
 - Deps: @aix/core, @aix/shared

 Step 3.2: @aix/mesh

 - Port crates/rlmx-mesh/src/ (1,859 LOC) → TypeScript
 - PersonalMesh, DiscoveryService (mdns-js for mDNS, ws for WebSocket relay)
 - SyncProtocol, FailoverPolicy
 - Deps: @aix/core, @aix/shared, mdns-js, ws

 Step 3.3: @aix/federation

 - Port crates/rlmx-federation/src/ (1,913 LOC) → TypeScript
 - FederationCycle orchestration in TS
 - Privacy operations call @aix/core.napiFederatedAnonymize()
 - Deps: @aix/core, @aix/shared

 Step 3.4: @aix/marketplace

 - Port crates/rlmx-marketplace/src/ (2,649 LOC) → TypeScript
 - Marketplace, AgentRegistry, BillingEngine, ReviewPipeline, FeaturedEngine
 - Deps: @aix/core, @aix/shared

 Phase 4: Composition packages

 Step 4.1: @aix/swarm

 - Port crates/rlmx-swarm/src/ (5,242 LOC) → TypeScript
 - Orchestration (zones, sandboxes, fleet) in TS
 - Consensus verification calls @aix/core.napiDispatch() for SHA-256
 - Deps: @aix/core, @aix/agents, @aix/shared

 Step 4.2: @aix/mcp-server

 - Port crates/rlmx-mcp/src/ (5,923 LOC) → TypeScript
 - Use official @modelcontextprotocol/sdk (Server class, tool registration)
 - 47 tool definitions as MCP SDK tool handlers (reference: crates/rlmx-mcp/src/tools.rs)
 - RBAC middleware (6 roles) wrapping SDK handlers
 - WebSocket event broadcasting for SwarmEvents (12 variants)
 - Tool dispatch calls @aix/core for kernel operations
 - Deps: @aix/core, @aix/agents, @aix/shared, @modelcontextprotocol/sdk, ws

 Phase 5: Top-level CLI

 Step 5.1: aix CLI package

 - Port crates/rlmx-cli/src/main.rs (2,429 LOC) → TypeScript
 - Commander.js commands mirroring all subcommands:
 aix serve --port 3000          # → @aix/mcp-server
 aix query -i "term"            # → @aix/core.napiDispatch
 aix voice start                # → @aix/core.napiVoicePipelineCreate
 aix agent spawn --type worker  # → @aix/agents
 aix marketplace search         # → @aix/marketplace
 aix mesh status                # → @aix/mesh
 aix billing status             # → @aix/billing
 aix federation status          # → @aix/federation
 - Deps: all @aix/* packages, commander, chalk, ora

 Build Commands

 # Root package.json scripts:
 "build:native": "cd crates/rlmx-napi && npx napi build --platform --release --features napi",
 "build:wasm": "wasm-pack build crates/rlmx-wasm --target web --out-dir ../../packages/wasm --out-name
 aix_wasm -- --features wasm",
 "build:ts": "npm run build --workspaces",
 "build": "npm run build:native && npm run build:wasm && npm run build:ts",
 "test:rust": "cargo test --workspace",
 "test:ts": "npm test --workspaces",
 "test": "npm run test:rust && npm run test:ts"

 Critical Files to Modify

 ┌─────────────────────────────┬──────────────────────────────────────────────────────────────────────┐
 │            File             │                                Change                                │
 ├─────────────────────────────┼──────────────────────────────────────────────────────────────────────┤
 │ crates/rlmx-napi/Cargo.toml │ Add deps: rlmx-voice, rlmx-phone, rlmx-cognitive, rlmx-rvf, rlmx-trm │
 ├─────────────────────────────┼──────────────────────────────────────────────────────────────────────┤
 │ crates/rlmx-napi/src/lib.rs │ Add ~25 NAPI functions for voice/phone/cognitive/rvf                 │
 ├─────────────────────────────┼──────────────────────────────────────────────────────────────────────┤
 │ Cargo.toml (workspace)      │ Remove archived crates from members, keep 9 Rust crates              │
 ├─────────────────────────────┼──────────────────────────────────────────────────────────────────────┤
 │ /rlmx/package.json          │ New root workspace config                                            │
 ├─────────────────────────────┼──────────────────────────────────────────────────────────────────────┤
 │ /rlmx/tsconfig.base.json    │ Shared TS compiler options                                           │
 ├─────────────────────────────┼──────────────────────────────────────────────────────────────────────┤
 │ packages/ (new dir)         │ All 16 npm packages                                                  │
 ├─────────────────────────────┼──────────────────────────────────────────────────────────────────────┤
 │ CLAUDE.md                   │ Update to reflect new architecture (9 Rust crates + 13 npm packages) │
 └─────────────────────────────┴──────────────────────────────────────────────────────────────────────┘

 Archive Process

 When each TypeScript package is complete and tested:
 1. Move crates/rlmx-<name>/ → crates/archive/rlmx-<name>/
 2. Remove from Cargo.toml workspace members
 3. Update any remaining Rust crates that depended on it (should be none after port)
 4. Run cargo build --workspace to verify clean build with fewer members

 Rust workspace after archive (9 crates):
 rlmx-kernel, rlmx-cognitive, rlmx-ruvllm, rlmx-trm, rlmx-rvf, rlmx-voice, rlmx-phone, rlmx-napi,
 rlmx-wasm

 Verification

 1. cargo build --workspace — 9 remaining Rust crates build clean
 2. cargo test --workspace — Rust tests pass (count will decrease as crates archive)
 3. npm run build:native — NAPI binary compiles for current platform
 4. npm run build:wasm — WASM module compiles
 5. npm run build:ts — All TypeScript packages compile
 6. npx aix --help — CLI shows all subcommands
 7. npx aix serve --port 3000 — MCP server starts
 8. npx aix voice start — Voice pipeline starts (via NAPI)
 9. npx aix marketplace search --domain finance — Marketplace works
 10. npm test --workspaces — All TypeScript tests pass
 11. Each archived Rust crate's tests are ported to its TS replacement

 Constraints

 - Voice and phone stay in Rust, exposed only via @aix/core NAPI
 - Follow ruvector pattern: per-platform optionalDependencies, JS loader, .d.ts types
 - @aix namespace for all packages, aix for top-level CLI
 - Graceful degradation: TS packages return { status: "unavailable" } when @aix/core fails to load
 - Archived Rust crates kept in crates/archive/ for reference, not compiled