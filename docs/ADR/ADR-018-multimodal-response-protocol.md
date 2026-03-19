# ADR-018: Multimodal Response Protocol

| Field    | Value                     |
|----------|---------------------------|
| Status   | Implemented               |
| Date     | 2026-03-19                |
| Authors  | Cedric                    |
| Replaces | —                         |

## Context

The existing `SwarmEvent` enum (ADR-008) has 9 variants for WebSocket real-time events. The voice-first PRD requires multimodal responses: voice narrating findings while visual cards render, haptic pulses indicate events, and progress indicators show per-domain completion. This requires expanding `SwarmEvent` with 3 new variants and defining a structured multimodal response protocol that the phone UI can render progressively.

## Decision

### 3 New SwarmEvent Variants

Expand `SwarmEvent` from 9 to 12 variants:

```rust
pub enum SwarmEvent {
    // Existing 9 variants (unchanged)
    NodeJoined { node_id: Uuid, zone: ZoneId, capabilities: Vec<String> },
    NodeLeft { node_id: Uuid, reason: String },
    ConsensusReached { topic: String, result: Value },
    TaskAssigned { task_id: Uuid, agent_id: Uuid, description: String },
    TaskCompleted { task_id: Uuid, result: Value, duration_ms: u64 },
    HealthCheck { node_id: Uuid, status: HealthStatus, metrics: Value },
    MetricsUpdate { node_id: Uuid, metrics: Value },
    SandboxEvent { sandbox_id: Uuid, event_type: String, payload: Value },
    FleetEvent { fleet_id: String, event_type: String, payload: Value },

    // New multimodal variants
    VoiceChunk {
        session_id: Uuid,
        audio_segment: Vec<u8>,       // Opus-encoded audio chunk
        transcript: String,            // Real-time partial transcript
        confidence: f32,               // STT confidence (0-1)
        is_final: bool,                // Final transcript for this segment
    },
    AgentProgress {
        task_id: Uuid,
        agent_type: String,            // AgentType variant name
        domain: String,                // Life domain category
        progress_pct: f32,             // 0.0 - 1.0
        status_text: String,           // Human-readable: "Scanning 12 arrondissements..."
        eta_ms: Option<u64>,           // Estimated time remaining
        partial_result: Option<Value>, // Early results for progressive rendering
    },
    MultimodalResponse {
        session_id: Uuid,
        voice_chunk: Option<Vec<u8>>,         // Streaming TTS audio (Opus)
        visual_card: Option<CardData>,        // Structured visual data
        haptic_pattern: Option<HapticPattern>,// Haptic feedback instruction
        is_final: bool,                        // Last chunk of response
    },
}
```

### CardData Structure

Visual cards are the primary visual response unit:

```rust
pub struct CardData {
    card_type: CardType,        // Summary, Comparison, Timeline, Action, Progress
    domain: String,             // Life domain for styling
    title: String,
    body: CardBody,             // Structured content
    actions: Vec<CardAction>,   // Buttons: "Start", "Compare", "Dismiss"
    priority: u8,               // Rendering order (0=highest)
}

pub enum CardBody {
    Text(String),
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    Comparison { items: Vec<ComparisonItem> },
    Timeline { events: Vec<TimelineEvent> },
    Chart { chart_type: String, data: Value },
    Metric { value: String, label: String, trend: f32 },
}

pub enum CardType {
    Summary,      // Executive overview
    Comparison,   // Side-by-side comparison (insurance plans, housing)
    Timeline,     // Sequential steps (moving timeline)
    Action,       // Single action with CTA button
    Progress,     // Domain progress indicator
}
```

### HapticPattern

```rust
pub enum HapticPattern {
    SavingsFound,    // Gentle pulse (money saved)
    UrgentAlert,     // Double-tap (security, fraud)
    TaskComplete,    // Long subtle buzz (agent finished)
    Notification,    // Single light tap
    Custom(Vec<HapticSegment>),
}

pub struct HapticSegment {
    intensity: f32,  // 0-1
    duration_ms: u16,
    pause_ms: u16,
}
```

### Response Mode Adaptation

The protocol adapts to the user's current context:

| Mode | Voice | Cards | Haptics | When |
|------|-------|-------|---------|------|
| Multimodal | Full TTS | Full cards | All patterns | Phone in hand, screen on |
| VoiceOnly | Full TTS | None | Urgent only | Driving, AirPods, screen off |
| Visual | None | Full cards | All patterns | Text preference, quiet environment |
| Ambient | None | Lock screen summary only | Urgent only | Phone locked, background |

Mode detected by: screen state, motion sensors (driving detection), audio output route (speaker vs. headphones), user preference override.

### WebSocket Protocol Extension

Multimodal events stream over existing WebSocket (ADR-008, port 3001):

- Binary frames for audio chunks (Opus codec, 48kHz mono)
- JSON frames for CardData and AgentProgress
- Interleaved: voice chunks and visual cards arrive on the same connection
- Client-side rendering: voice plays immediately, cards accumulate and render in priority order

### Progressive Rendering Pipeline

1. User speaks → VoiceChunk events stream partial transcript
2. Router decides strategy → AgentProgress events begin (one per agent)
3. Agents execute → AgentProgress updates with progress_pct and partial_results
4. First domain completes → MultimodalResponse with voice_chunk (TTS narrates finding) + visual_card
5. More domains complete → more MultimodalResponse events, cards accumulate
6. All domains complete → final MultimodalResponse with is_final=true, summary card

### Backpressure

- Voice chunks: drop-oldest policy (audio must be real-time)
- Cards: queue with priority ordering (high-priority cards rendered first)
- AgentProgress: latest-wins per task_id (only most recent progress matters)
- Client sends flow control hints: "pause_voice" when user is reading cards

## Consequences

### Positive

- Rich, responsive user experience with voice + visual + haptic feedback
- Progressive rendering eliminates "staring at loading spinner" feeling
- Context-aware mode adaptation works seamlessly across usage scenarios
- Reuses existing WebSocket infrastructure (ADR-008)

### Negative

- Binary audio frames on WebSocket add complexity to client parsing
- Haptic patterns require platform-specific native bridge code (iOS/Android)
- Mode detection heuristics may misclassify (e.g., phone in pocket → ambient, but user has AirPods)

### Risks

- Audio/visual sync issues if network is jittery
- Opus codec licensing for commercial use (royalty-free, but must verify)
- Card rendering performance on low-end phones with many concurrent agents

## References

- ADR-008: WebSocket Real-Time Events (existing SwarmEvent, port 3001)
- ADR-012: Voice-First Pipeline (TTS streaming)
- ADR-015: Multi-Intent Fan-Out (AgentProgress streaming)
- PRD Section 2: Multimodal Response System
- PRD Section 3: New SwarmEvent Variants

## Implementation Notes

Implemented in `crates/rlmx-mcp/src/ws.rs` and `crates/rlmx-swarm/src/types.rs`. SwarmEvent expanded from 9 to 12 variants: VoiceChunk (streaming transcript), AgentProgress (per-domain progress), MultimodalResponse (voice + visual cards + haptics). CardData struct and HapticPattern enum (Gentle/DoubleTap/LongBuzz/Alert/Success/Warning). 3 voice MCP tools added (rlmx_voice_transcribe, rlmx_voice_synthesize, rlmx_voice_session).
