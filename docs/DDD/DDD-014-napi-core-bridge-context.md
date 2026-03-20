# DDD-014: NAPI Core Bridge Bounded Context

## Overview
The NAPI Core Bridge context serves as the anti-corruption layer between the Rust compute kernel and the TypeScript orchestration layer. It translates TypeScript function calls into Rust struct method invocations, manages Rust-side state (voice sessions, phone runtime, cognitive models), and enforces type safety across the FFI boundary. This context expands the existing NapiKernel (ADR-020) from 6 to ~31 functions to support the TypeScript migration (ADR-026).

## Aggregate Root: NapiKernel

### Identity
- `NapiKernelInstance` — Singleton per Node.js process (created via `NapiKernel::new()`)
- No external ID — identity is the process itself

### Entities
- **KernelBridge**: Wraps `rlmx-kernel::KernelContext` for syscall dispatch
  - `kernel: Arc<Mutex<KernelContext>>`
  - Existing 6 functions: dispatch, sona_query, rvf_seal, agent_spawn, swarm_status, route

- **VoiceBridge**: Wraps `rlmx-voice::VoicePipeline` for voice operations
  - `pipeline: Option<Arc<Mutex<VoicePipeline>>>`
  - `sessions: Arc<RwLock<HashMap<Uuid, VoiceSession>>>`
  - 6 functions: pipeline_create, vad_process, transcribe, decompose_intents, synthesize, session_create

- **PhoneBridge**: Wraps `rlmx-phone::PhoneRuntime` for engagement and scheduling
  - `runtime: Option<Arc<Mutex<PhoneRuntime>>>`
  - 8 functions: runtime_create, engagement_score, savings_record, streak_check_in, streak_status, battery_policy, coordinator_status, notification_send

- **CognitiveBridge**: Wraps `rlmx-cognitive::Sona` and `VoicePatternBank`
  - `sona: Arc<Mutex<Sona>>`
  - `pattern_bank: Arc<RwLock<VoicePatternBank>>`
  - 5 functions: sona_record, sona_adapt, voice_pattern_search, federated_anonymize, fatigue_check

- **InferenceBridge**: Wraps `rlmx-ruvllm::TieredEngine` (feature-gated)
  - `engine: Option<Arc<Mutex<TieredEngine>>>`
  - 5 functions: load, generate, status, tiered_route, unload

### Value Objects
- `NapiError` — Structured error with code and message, converted from Rust errors at FFI boundary
- `NapiConfig` — Initialization configuration (model paths, device capabilities, feature flags)
- `FunctionGroup` — Enum categorizing NAPI functions: Kernel, Voice, Phone, Cognitive, Rvf, Inference

### Invariants
1. NapiKernel is a singleton — only one instance per Node.js process
2. All Rust state is behind `Arc<Mutex<T>>` or `Arc<RwLock<T>>` — thread-safe across NAPI worker threads
3. Voice pipeline must be created before calling voice functions (returns `VOICE_NOT_INITIALIZED` otherwise)
4. Phone runtime must be created before calling phone functions (returns `PHONE_NOT_INITIALIZED` otherwise)
5. Inference functions require `ruvllm` feature flag at compile time — stub returns `INFERENCE_UNAVAILABLE`
6. Audio data never crosses the NAPI boundary as raw samples — only metadata and text transcripts
7. All NAPI functions are async (`napi::Result<T>`) — Rust panics caught and converted to JS errors
8. Session cleanup: voice sessions expire after 30 minutes of inactivity
9. Existing 6 functions maintain backward compatibility — no signature changes
10. Type conversions between JS and Rust are lossless for all supported types (numbers, strings, buffers, arrays)

## Domain Events
| Event | Trigger | Consumers |
|-------|---------|-----------|
| `BridgeInitialized` | `NapiKernel::new()` called | Logging, health check |
| `VoicePipelineCreated` | `napiVoicePipelineCreate()` | Voice session management |
| `VoiceSessionCreated` | `napiVoiceSessionCreate()` | Session tracking, timeout scheduler |
| `VoiceSessionExpired` | 30-min inactivity timeout | Session cleanup |
| `PhoneRuntimeCreated` | `napiPhoneRuntimeCreate()` | Engagement tracking |
| `InferenceModelLoaded` | `napiInferenceLoad()` | Memory tracking, routing |
| `InferenceModelUnloaded` | `napiInferenceUnload()` | Memory reclaim |
| `NapiError` | Any function failure | Error telemetry |

## Commands
- `InitializeKernel(config: NapiConfig)` — Create NapiKernel singleton with all bridges
- `CreateVoicePipeline(config: VoicePipelineConfig)` — Initialize voice bridge
- `CreatePhoneRuntime(device_id, capabilities)` — Initialize phone bridge
- `LoadInferenceModel(model_path, tier)` — Load model into inference bridge
- `CleanupExpiredSessions()` — Remove stale voice sessions (called by timer)

## Queries
- `GetBridgeStatus()` — Which bridges are initialized, function counts, session counts
- `GetFunctionInventory()` — List all available NAPI functions by group
- `GetSessionCount()` — Active voice session count

## Anti-Corruption Layer
- **To Kernel Syscall Context (DDD-001)**: KernelBridge wraps KernelContext — translates JS objects to Rust `SyscallPermission` enums, returns JSON-serializable results
- **To Voice Interaction Context (DDD-008)**: VoiceBridge wraps VoicePipeline — converts JS audio metadata to Rust `AudioFrame`, returns `VadDecision`/`Intent` as plain objects
- **To Phone Runtime Context (DDD-009)**: PhoneBridge wraps PhoneRuntime — maps JS engagement calls to Rust `LifeScore`/`StreakState`, enforces engagement invariants in Rust
- **To Observation & Health Context (DDD-006)**: CognitiveBridge wraps Sona — Float32Array embeddings cross the boundary, SONA adaptation runs entirely in Rust
- **To Container & Storage Context (DDD-007)**: RVF seal and verify operate on Rust-side Ed25519 — JS receives only verification results
- **To Inference Routing Context (DDD-004)**: InferenceBridge wraps TieredEngine — model loading and generation happen in Rust, JS receives text output

### Translation Rules
| JS Type | Rust Type | Direction |
|---------|-----------|-----------|
| `string` | `String` | Bidirectional |
| `number` | `f64` / `u64` / `i64` | Bidirectional (precision noted in .d.ts) |
| `Float32Array` | `Vec<f32>` | JS → Rust (embeddings, vectors) |
| `Buffer` | `Vec<u8>` | Bidirectional (binary data) |
| `object` | `serde_json::Value` | Bidirectional (complex types) |
| `Promise<T>` | `napi::Result<T>` | Rust → JS (all async) |

## Crate Mapping
- Primary: `rlmx-napi` (expanded)
- Dependencies: `rlmx-kernel`, `rlmx-voice`, `rlmx-phone`, `rlmx-cognitive`, `rlmx-rvf`, `rlmx-trm`, `rlmx-ruvllm` (optional)
- Output: `@aix/core` npm package + per-platform binary packages

## Ubiquitous Language Additions
- **Bridge**: A wrapper around a Rust crate's aggregate root, exposed via NAPI functions
- **Function Group**: A category of related NAPI functions (Voice, Phone, Cognitive, etc.)
- **FFI Boundary**: The Rust ↔ JavaScript interface where type translation occurs
- **Bridge Initialization**: Creating the Rust-side state for a function group (lazy, on first use)
- **Session Timeout**: Automatic cleanup of voice sessions after 30 minutes of inactivity
- **Stub Return**: The response when a feature-gated bridge is called without the feature enabled
