# ADR-027: NAPI Bridge Expansion

Status: Proposed

## Context
The existing rlmx-napi crate (ADR-020) exposes 6 NAPI functions: `dispatch`, `sona_query`, `rvf_seal`, `agent_spawn`, `swarm_status`, and `route`. The TypeScript migration (ADR-026) requires TypeScript packages to call into Rust compute for voice processing, phone engagement, cognitive operations, and RVF verification. Rather than maintaining multiple NAPI crates, we expand the single `@aix/core` binary to expose all Rust compute operations.

## Decision
Expand rlmx-napi from 6 to ~31 NAPI functions, adding voice (6), phone (8), cognitive (5), RVF (1), and inference (5) function groups. The `NapiKernel` aggregate root grows to hold `VoicePipeline`, `PhoneRuntime`, `Sona`, `VoicePatternBank`, and session state.

### New NAPI Dependencies

| Crate | Functions Exposed | Why NAPI (not TS) |
|-------|-------------------|-------------------|
| rlmx-voice | 6 | VAD signal processing, STT model inference, TTS synthesis |
| rlmx-phone | 8 | Engagement scoring, battery-aware scheduling, offline outbox |
| rlmx-cognitive | 5 | SONA micro-LoRA adaptation, pattern bank search, fatigue model |
| rlmx-rvf | 1 | Ed25519 witness verification |
| rlmx-ruvllm | 5 | Candle inference, model loading, Metal/CUDA dispatch |

### Function Inventory

**Existing (unchanged)**:
```typescript
// Kernel operations
napiDispatch(syscall: string, args: object): Promise<object>
napiSonaQuery(query: string): Promise<object>
napiRvfSeal(data: Buffer): Promise<object>
napiAgentSpawn(agentType: string, config: object): Promise<object>
napiSwarmStatus(): Promise<object>
napiRoute(input: Float32Array): Promise<object>
```

**Voice (6 new)**:
```typescript
napiVoicePipelineCreate(config: VoicePipelineConfig): Promise<string>
napiVoiceVadProcess(energy: number, spectral: number, durationMs: number): Promise<VadDecision>
napiVoiceTranscribe(audioLen: number, language: string, tier: string): Promise<TranscriptResult>
napiVoiceDecomposeIntents(transcript: string, maxIntents: number): Promise<Intent[]>
napiVoiceSynthesize(text: string, persona: string, streaming: boolean): Promise<TtsResult>
napiVoiceSessionCreate(): Promise<string>
```

**Phone (8 new)**:
```typescript
napiPhoneRuntimeCreate(deviceId: string, capabilities: DeviceCapabilities): Promise<string>
napiPhoneEngagementScore(): Promise<EngagementScore>
napiPhoneSavingsRecord(cents: number, domain: string, proof: string): Promise<SavingsResult>
napiPhoneStreakCheckIn(): Promise<StreakState>
napiPhoneStreakStatus(): Promise<StreakState>
napiPhoneBatteryPolicy(level: number, thermal: string): Promise<BatteryPolicy>
napiPhoneCoordinatorStatus(): Promise<CoordinatorStatus>
napiPhoneNotificationSend(title: string, body: string, priority: string): Promise<NotificationResult>
```

**Cognitive (5 new)**:
```typescript
napiSonaRecord(query: string, actions: string[], quality: number): Promise<void>
napiSonaAdapt(embedding: Float32Array, quality: number): Promise<AdaptResult>
napiVoicePatternSearch(query: string, topK: number): Promise<PatternResult[]>
napiFederatedAnonymize(patternJson: string): Promise<string>
napiFatigueCheck(userId: string): Promise<FatigueResult>
```

**RVF (1 new)**:
```typescript
napiRvfVerify(witnessId: string): Promise<VerifyResult>
```

**Inference (5 new)**:
```typescript
napiInferenceLoad(modelPath: string, tier: string): Promise<string>
napiInferenceGenerate(prompt: string, maxTokens: number): Promise<GenerateResult>
napiInferenceStatus(): Promise<InferenceStatus>
napiInferenceTieredRoute(query: string, confidence: number): Promise<RouteDecision>
napiInferenceUnload(modelId: string): Promise<void>
```

### NapiKernel State Expansion

```rust
pub struct NapiKernel {
    // Existing
    kernel: Arc<Mutex<KernelContext>>,

    // New: voice pipeline
    voice_pipeline: Option<Arc<Mutex<VoicePipeline>>>,
    voice_sessions: Arc<RwLock<HashMap<Uuid, VoiceSession>>>,

    // New: phone runtime
    phone_runtime: Option<Arc<Mutex<PhoneRuntime>>>,

    // New: cognitive
    sona: Arc<Mutex<Sona>>,
    voice_pattern_bank: Arc<RwLock<VoicePatternBank>>,

    // New: inference
    tiered_engine: Option<Arc<Mutex<TieredEngine>>>,
}
```

### Cargo.toml Changes

```toml
[dependencies]
rlmx-kernel = { path = "../rlmx-kernel" }
# Existing ^^

# New dependencies
rlmx-voice = { path = "../rlmx-voice" }
rlmx-phone = { path = "../rlmx-phone" }
rlmx-cognitive = { path = "../rlmx-cognitive" }
rlmx-rvf = { path = "../rlmx-rvf" }
rlmx-trm = { path = "../rlmx-trm" }
rlmx-ruvllm = { path = "../rlmx-ruvllm", optional = true }
```

### Binary Size Impact
- Current `@aix/core` binary: ~5MB per platform
- Projected with voice+phone+cognitive+rvf: ~15-20MB per platform
- Metal/CUDA features remain opt-in via cargo features
- Per-platform npm packages keep install size minimal (only download target platform)

### Error Handling
All NAPI functions return `napi::Result<T>`. Rust panics are caught at the FFI boundary and converted to JavaScript errors with structured error codes:

```typescript
try {
  const result = await core.napiVoiceTranscribe(audioLen, 'en', 'small');
} catch (e) {
  // e.code: 'VOICE_ENGINE_UNAVAILABLE' | 'MODEL_NOT_LOADED' | ...
  // e.message: human-readable description
}
```

## Consequences

### Positive
- Single `@aix/core` binary for all Rust compute — simpler distribution
- TypeScript packages call native code without HTTP overhead (sub-ms latency)
- Voice privacy invariant preserved — audio processing stays in Rust/NAPI, never in TS
- Existing 6 functions unchanged — no breaking changes for current NAPI users

### Negative
- Binary size increases ~3-4x (5MB → 15-20MB per platform)
- NapiKernel becomes a larger struct with more initialization complexity
- 31 functions to maintain type alignment between Rust and TypeScript

### Risks
- Voice/phone state management in NAPI process (mitigated: Arc<Mutex> pattern, same as kernel)
- Long-running voice sessions may leak memory (mitigated: session timeout + explicit cleanup)
- Platform-specific inference features (Metal/CUDA) require conditional compilation (mitigated: existing feature gate pattern from ADR-024)

## References
- ADR-020: NAPI-RS Native Bindings (original 6 functions)
- ADR-026: TypeScript Migration Strategy (why expansion is needed)
- ADR-012: Voice-First Pipeline (voice function semantics)
- ADR-013: Phone Command Center (phone function semantics)
- DDD-014: NAPI Core Bridge Context (bounded context design)
