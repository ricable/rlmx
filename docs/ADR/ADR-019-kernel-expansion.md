# ADR-019: Kernel Expansion for Voice-First Architecture

| Field    | Value                     |
|----------|---------------------------|
| Status   | Implemented               |
| Date     | 2026-03-19                |
| Authors  | Cedric                    |
| Replaces | —                         |

## Context

The RLMX cognition kernel (rlmx-kernel) currently provides 12 syscalls, 6 domain events, and a 14-dimensional TinyDancerRouter. The voice-first PRD requires expanding these to support voice transcription, voice synthesis, intent routing, and richer event propagation. Additionally, the `SandboxProfile` (ADR-011) needs mobile-aware fields, and the MCP server needs 8 new marketplace tools (ADR-014).

## Decision

### Syscall Expansion: 12 → 15

Add 3 new syscalls to the existing dispatch table:

```rust
pub enum Syscall {
    // Existing 12 (unchanged)
    VecSearch,
    VecInsert,
    StateMutate,
    GraphQuery,
    ProcessFork,
    ProcessSend,
    ProcessHalt,
    HaltCheck,
    ProofSeal,
    ProofVerify,
    ProofWitness,
    StrategyRoute,

    // New voice syscalls
    VoiceTranscribe {
        audio: AudioBuffer,
        language_hint: Option<String>,
        tier_override: Option<ModelTier>,  // Force specific tier
    },
    VoiceSynthesize {
        text: String,
        persona: VoicePersona,            // 1 of 6 domain personas
        streaming: bool,                   // Stream chunks or wait for full
        rate: Option<f32>,                 // Speech rate multiplier
    },
    IntentRoute {
        transcript: String,
        speaker_context: SpeakerContext,   // Speaker embedding, history
        decompose: bool,                   // true = multi-intent decomposition
        max_intents: Option<usize>,        // Cap on decomposed intents
    },
}
```

### Permission Matrix Update

The 12x12 permission matrix (ADR-005) expands to 12x15:
- All agent types get `IntentRoute` permission (read-only, no side effects)
- `VoiceTranscribe` restricted to Coordinator and Router agent types
- `VoiceSynthesize` available to all agent types (agents can speak to user)

### Domain Event Expansion: 6 → 8

```rust
pub enum DomainEvent {
    // Existing 6 (unchanged)
    SyscallDispatched { syscall: Syscall, process: ProcessId },
    QueryRouted { strategy: Strategy, confidence: f32 },
    StateChanged { key: String, version: u64 },
    ProofSealed { proof_id: Uuid },
    AgentSpawned { agent_id: Uuid, agent_type: AgentType },
    AgentTerminated { agent_id: Uuid, reason: String },

    // New voice events
    VoiceSessionStarted {
        session_id: Uuid,
        speaker_id: Option<Uuid>,     // On-device speaker identification
        mode: ResponseMode,            // VoiceOnly, Visual, Multimodal, Ambient
    },
    VoiceResponseStreaming {
        session_id: Uuid,
        domain: String,
        chunk_index: u32,
        is_final: bool,
    },
}
```

### TinyDancerRouter Expansion: 14 → 18 Dimensions

The FastGRNN architecture changes from 14→32→5 to 18→32→5:

```rust
pub struct RouterInput {
    // Original 14 dimensions
    pub query_complexity: f32,
    pub token_count: f32,
    pub domain_signals: [f32; 5],      // 5 domain classifiers
    pub battery_level: f32,
    pub network_type: f32,             // 0=offline, 0.5=cellular, 1=wifi
    pub time_of_day: f32,              // Normalized 0-1
    pub model_confidence_history: [f32; 3],  // Last 3 routing outcomes
    pub memory_pressure: f32,

    // New voice-aware dimensions
    pub speaker_confidence: f32,       // STT confidence score (0-1)
    pub emotion_valence: f32,          // Detected emotion (-1 to 1)
    pub urgency_score: f32,            // Speech rate + keyword triggers (0-1)
    pub ambient_noise_level: f32,      // Background noise estimate (normalized dB)
}
```

Weight initialization for new dimensions: zero-init (no impact until online SGD trains from voice data). Backward-compatible: text-only queries pass 0.0 for all 4 voice dims.

### Voice-Aware Routing Rules (Learned via Online SGD)

| Condition | Routing Decision |
|-----------|-----------------|
| Low STT confidence (< 0.7) + high urgency | Escalate to cloud STT before routing |
| High emotion valence (anger/stress > 0.7) | Prioritize speed: route to fastest available engine |
| High ambient noise (> 0.8) | Increase TTS volume, simplify response (voice-only mode) |
| Low battery (< 0.15) + voice query | Compress response, skip visual cards |
| High urgency + finance domain | Route to cloud for fastest response (skip device inference) |

These rules emerge from online SGD, not hardcoded — the router learns per-user patterns within ~100 voice interactions.

### SandboxProfile Expansion (ADR-011 Amendment)

Add 5 new fields to `SandboxProfile`:

```rust
pub struct SandboxProfile {
    // Existing fields (unchanged)
    pub name: String,
    pub agent_type: AgentType,
    pub resources: ResourceEnvelope,
    pub network: NetworkPolicy,
    pub zone_preference: String,
    pub model_tier: String,
    pub crate_features: Vec<String>,
    pub rvf_payload: Option<String>,

    // New mobile-aware fields
    pub device_class: DeviceClass,           // Phone, Tablet, Laptop, Pi, Cloud
    pub background_policy: BackgroundPolicy, // Foreground, Background, Suspended
    pub battery_threshold: f32,              // Min battery % to run (0.0-1.0)
    pub network_requirement: NetworkRequirement, // Any, WiFi, Offline
    pub thermal_budget: ThermalBudget,       // Low, Medium, High
}

pub enum DeviceClass { Phone, Tablet, Laptop, Pi, Cloud }
pub enum BackgroundPolicy { Foreground, Background, Suspended }
pub enum NetworkRequirement { Any, WiFi, Offline }
pub enum ThermalBudget { Low, Medium, High }
```

### MCP Tool Expansion: 28 → 36

8 new tools added (see ADR-014 for marketplace details):
1. `rlmx_marketplace_search`
2. `rlmx_marketplace_install`
3. `rlmx_marketplace_uninstall`
4. `rlmx_marketplace_rate`
5. `rlmx_marketplace_publish`
6. `rlmx_marketplace_earnings`
7. `rlmx_marketplace_featured`
8. `rlmx_marketplace_categories`

All tools follow existing JSON-RPC 2.0 protocol (ADR-010), require RBAC role `User` or above, and return standardized response format.

### Migration Strategy

1. **Backward compatible**: New syscalls are additive. Existing code continues to work.
2. **Feature-gated**: Voice syscalls behind `voice` feature flag. `cargo build --workspace` (no features) compiles clean.
3. **Router migration**: Load existing 14-dim weights into first 14 dims of 18-dim router. Zero-init remaining 4. No retraining needed — online SGD adapts gradually.
4. **Event bus**: New DomainEvent variants are non-breaking additions to the enum.
5. **SandboxProfile**: New fields have sensible defaults (DeviceClass::Cloud, BackgroundPolicy::Foreground, etc.).

## Consequences

### Positive
- Kernel gains voice-native capabilities without breaking existing 12-syscall API
- Router becomes context-aware (emotion, urgency, noise) for dramatically better routing decisions
- Feature-gated approach maintains stub build integrity
- 36 MCP tools cover complete voice + marketplace + existing functionality

### Negative
- 15 syscalls x 12 agent types = larger permission matrix to maintain
- 18-dim router requires more training data to converge than 14-dim
- 3 new crates (voice, phone, marketplace) increase workspace build time

### Risks
- Voice feature flag may create two divergent build paths
- Router weight migration (14 to 18 dim) could temporarily degrade routing quality
- SandboxProfile field explosion (now 13 fields) may need restructuring later

## References
- ADR-003: Neural Model Routing (TinyDancerRouter original design)
- ADR-005: Capability-Secured Agents (permission matrix)
- ADR-008: WebSocket Real-Time Events (DomainEvent, SwarmEvent)
- ADR-010: MCP Tool Expansion (existing 28 tools)
- ADR-011: Sandbox Orchestration (SandboxProfile)
- ADR-014: Agent Marketplace (8 new MCP tools)
- PRD Section 11: Technical Architecture

## Implementation Notes

Implemented in `crates/rlmx-kernel/`. Syscalls expanded 12->15 (VoiceTranscribe, VoiceSynthesize, IntentRoute). DomainEvents expanded 6->8 (VoiceSessionStarted, IntentsDecomposed). TinyDancerRouter expanded 14->18 input dimensions (+speaker_confidence, emotion_valence, urgency_score, ambient_noise_level). New types: ResponseMode (4 variants), VoicePersona (6 variants), LifeDomain (12 variants), Intent struct. SyscallPermission expanded to 15. Permission matrix updated to 15x14 in rlmx-agents. AgentType expanded 12->14 (VoiceCoordinator, MarketplaceManager).
