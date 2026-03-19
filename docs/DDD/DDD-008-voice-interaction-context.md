# DDD-008: Voice Interaction Bounded Context

## Overview

The Voice Interaction context owns the complete voice pipeline: always-listening
VAD, on-device STT, multi-intent decomposition, TTS with domain personas, and
voice session memory. It transforms raw audio into structured intents that the
Kernel Syscall context dispatches, and synthesizes multimodal responses back to
the user.

This context enforces strict privacy invariants: audio never leaves the device,
speaker embeddings are never federated, and emotion data is bucketed before
transmission. The pipeline is designed for sub-200ms first-audio-chunk latency
on mobile hardware.

**Crate**: `crates/rlmx-voice/`

## Aggregate Root: VoiceSession

```rust
pub struct VoiceSession {
    pub id: Uuid,
    pub speaker_id: Option<Uuid>,        // On-device speaker identification
    pub mode: ResponseMode,               // VoiceOnly, Visual, Multimodal, Ambient
    pub turns: Vec<ConversationTurn>,
    pub active_intents: Vec<Intent>,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub emotion_trajectory: Vec<f32>,     // Emotion valence over session
}
```

`VoiceSession` is the aggregate root because it owns the consistency boundary
for a single voice interaction lifecycle -- from wake word through response
delivery. All mutations to conversation turns, intent state, and emotion
tracking flow through the session. A session is the unit of resource allocation
(audio buffer, STT context, TTS stream) and the unit of cleanup on timeout.

## Entities

### ConversationTurn

Represents a single user-system exchange within a session:

```rust
pub struct ConversationTurn {
    pub id: Uuid,
    pub direction: TurnDirection,        // UserToSystem, SystemToUser
    pub transcript: String,
    pub audio_duration_ms: u64,
    pub intents: Vec<Intent>,
    pub response: Option<MultimodalResponse>,
    pub timestamp: DateTime<Utc>,
}

pub enum TurnDirection {
    UserToSystem,
    SystemToUser,
}
```

A turn is an entity (not a value object) because it has identity within the
session -- downstream consumers reference turns by ID for follow-up resolution
and context threading.

### VoicePipeline

The runtime pipeline that processes audio through the full voice stack:

```rust
pub struct VoicePipeline {
    pub vad: VoiceActivityDetector,
    pub stt: SpeechToText,
    pub intent_classifier: IntentClassifier,
    pub decomposer: MultiIntentDecomposer,
    pub tts: TextToSpeech,
    pub session_memory: SessionMemory,
}
```

The pipeline is an entity because its lifecycle is managed independently of
any single session -- it is initialized at app start and shared across sessions,
with mutable state in its VAD threshold calibration and STT language model
adaptation.

### MultimodalResponse

The synthesized response delivered back to the user:

```rust
pub struct MultimodalResponse {
    pub id: Uuid,
    pub voice_chunks: Vec<OpusChunk>,     // Streamed Opus audio
    pub visual: Option<VisualPayload>,    // Cards, charts, lists
    pub haptic: Option<HapticPattern>,    // Vibration feedback
    pub persona: VoicePersona,
    pub latency_to_first_chunk_ms: u64,
}

pub struct OpusChunk {
    pub index: u32,
    pub data: Vec<u8>,
    pub duration_ms: u16,
}

pub struct VisualPayload {
    pub card_type: String,
    pub title: String,
    pub body: String,
    pub actions: Vec<String>,
}

pub enum HapticPattern {
    Confirmation,
    Alert,
    Subtle,
}
```

## Value Objects

### Intent

```rust
pub struct Intent {
    pub domain: LifeDomain,              // 1 of 12 categories
    pub action: String,                  // "cancel", "search", "compare"
    pub entities: Vec<ExtractedEntity>,
    pub urgency: f32,                    // 0-1
    pub confidence: f32,                 // Decomposition confidence
}

pub struct ExtractedEntity {
    pub entity_type: String,             // "date", "amount", "location"
    pub value: String,
    pub span: (usize, usize),           // Character offsets in transcript
}
```

A single utterance may decompose into multiple intents spanning different life
domains (e.g., "Cancel my dentist appointment and order more dog food" yields
two intents in Health and Shopping domains).

### ResponseMode

```rust
pub enum ResponseMode {
    VoiceOnly,    // Driving, AirPods, screen off
    Visual,       // Text preference, quiet environment
    Multimodal,   // Phone in hand, screen on
    Ambient,      // Phone locked, background notifications
}
```

Response mode is inferred from device sensors (accelerometer, proximity,
screen state, audio output route) and can change mid-session.

### VoicePersona

```rust
pub struct VoicePersona {
    pub domain: LifeDomain,
    pub style: PersonaStyle,     // Warm, Authoritative, Upbeat, Calm, Neutral
    pub rate: f32,               // Speech rate multiplier (0.5-2.0)
    pub model_id: String,        // TTS model identifier
}

pub enum PersonaStyle {
    Warm,
    Authoritative,
    Upbeat,
    Calm,
    Neutral,
}
```

Each of the 12 life domains has a default persona. Users can override persona
selection per domain.

### SpeakerContext

```rust
pub struct SpeakerContext {
    pub speaker_embedding: Vec<f32>,    // On-device voice fingerprint
    pub confidence: f32,                 // Speaker ID confidence
    pub emotion_valence: f32,            // -1 to 1 (negative to positive)
    pub urgency_score: f32,              // 0-1
    pub ambient_noise_db: f32,
}
```

Speaker context is extracted per utterance and normalized before passing to
the router. The embedding vector stays on-device; only the scalar features
(confidence, emotion bucket, urgency, noise level) cross context boundaries.

### VoiceActivityDetector

```rust
pub struct VoiceActivityDetector {
    pub threshold: f32,                  // Adaptive silence threshold
    pub buffer_duration_ms: u64,         // Rolling buffer size (3000ms)
    pub wake_word_enabled: bool,
    pub wake_word_model: String,
}
```

## Value Object Summary

| Value Object | Definition |
|-------------|------------|
| `Intent` | Structured action extracted from transcript with domain, entities, confidence |
| `ResponseMode` | Enum: `VoiceOnly`, `Visual`, `Multimodal`, `Ambient` |
| `VoicePersona` | TTS voice configuration per life domain |
| `SpeakerContext` | Per-utterance speaker features (embedding, emotion, urgency, noise) |
| `PersonaStyle` | Enum: `Warm`, `Authoritative`, `Upbeat`, `Calm`, `Neutral` |
| `ExtractedEntity` | Named entity with type, value, and span offsets |
| `OpusChunk` | Single Opus-encoded audio chunk for streaming TTS |
| `HapticPattern` | Enum: `Confirmation`, `Alert`, `Subtle` |
| `LifeDomain` | 1 of 12 life categories (defined in Kernel, re-exported here) |

## Domain Events

| Event | Trigger | Data |
|-------|---------|------|
| `SessionStarted` | Wake word or manual activation | `{ session_id, speaker_id, mode }` |
| `SpeechDetected` | VAD crosses threshold | `{ session_id, vad_confidence }` |
| `TranscriptReady` | STT produces final transcript | `{ session_id, transcript, stt_confidence, is_final }` |
| `IntentsDecomposed` | Multi-intent classifier completes | `{ session_id, intents: Vec<Intent> }` |
| `ResponseStreaming` | First TTS chunk generated | `{ session_id, domain, chunk_index }` |
| `SessionEnded` | Timeout or explicit close | `{ session_id, turn_count, duration_ms }` |
| `ResponseModeChanged` | Sensor state change detected | `{ session_id, old_mode, new_mode }` |
| `SpeakerIdentified` | Speaker embedding matched | `{ session_id, speaker_id, confidence }` |

```rust
pub enum VoiceDomainEvent {
    SessionStarted {
        session_id: Uuid,
        speaker_id: Option<Uuid>,
        mode: ResponseMode,
    },
    SpeechDetected {
        session_id: Uuid,
        vad_confidence: f32,
    },
    TranscriptReady {
        session_id: Uuid,
        transcript: String,
        stt_confidence: f32,
        is_final: bool,
    },
    IntentsDecomposed {
        session_id: Uuid,
        intents: Vec<Intent>,
    },
    ResponseStreaming {
        session_id: Uuid,
        domain: String,
        chunk_index: u32,
    },
    SessionEnded {
        session_id: Uuid,
        turn_count: u32,
        duration_ms: u64,
    },
    ResponseModeChanged {
        session_id: Uuid,
        old_mode: ResponseMode,
        new_mode: ResponseMode,
    },
    SpeakerIdentified {
        session_id: Uuid,
        speaker_id: Uuid,
        confidence: f32,
    },
}
```

## Domain Services

### `detect_and_transcribe(audio_buffer) -> TranscriptResult`

The ingest path from raw audio to text:

1. VAD analyzes 3-second rolling buffer for speech onset/offset.
2. If speech detected and wake word confirmed (or manual trigger active),
   activate STT engine.
3. STT produces streaming partial transcripts, then a final transcript.
4. Emit `SpeechDetected` and `TranscriptReady` events.
5. Audio buffer is discarded after transcription -- never persisted.

### `decompose_intents(transcript, speaker_context) -> Vec<Intent>`

Multi-intent extraction from a single utterance:

1. Run intent classifier to identify life domains present in transcript.
2. For each domain, extract action and entities.
3. Assign urgency scores based on speaker emotion and linguistic markers.
4. Emit `IntentsDecomposed` event.
5. Return sorted by urgency (highest first).

### `synthesize_response(intents, results, mode) -> MultimodalResponse`

Response assembly and TTS synthesis:

1. Select persona for the primary intent's domain.
2. Generate text response from intent results.
3. Begin Opus-encoded TTS streaming (target: first chunk within 200ms).
4. If mode is `Multimodal` or `Visual`, generate visual payload in parallel.
5. Emit `ResponseStreaming` event on first chunk.

### `manage_session(session_id) -> SessionState`

Session lifecycle management:

1. Track turn history and active intents.
2. Monitor `last_activity` for 5-minute timeout enforcement.
3. Persist session summary to SONA `PatternBank` for long-term memory.
4. Emit `SessionEnded` on timeout or explicit close.

## Invariants

1. **No audio storage**: Audio is processed in a 3-second rolling buffer and
   never persisted. Only text transcripts and metadata flow downstream. This
   is enforced at the `VoiceActivityDetector` level -- the buffer is a ring
   buffer that overwrites continuously.

2. **Speaker embedding stays on-device**: Speaker identification embeddings
   are computed and stored only in on-device memory. They are never serialized
   to network, never included in domain events, and never written to federated
   storage.

3. **Wake word required before STT**: The STT engine only activates after
   VAD + wake word detection (or explicit manual trigger). This prevents
   continuous transcription and unnecessary compute.

4. **Emotion valence is bucketed before federation**: Continuous `f32` emotion
   valence is quantized to 5 discrete buckets (`VeryNegative`, `Negative`,
   `Neutral`, `Positive`, `VeryPositive`) before any data leaves the device.
   This is enforced at the anti-corruption layer boundary (see ADR-017).

5. **Session timeout at 5 minutes**: Sessions auto-close after 5 minutes of
   inactivity (`last_activity` check) to prevent resource leaks. The pipeline
   releases audio buffers, STT context, and TTS streams on session close.

6. **TTS streaming starts within 200ms**: The first Opus audio chunk must
   begin streaming within 200ms of response generation start. This is measured
   and tracked in `MultimodalResponse::latency_to_first_chunk_ms`.

7. **Intent confidence threshold**: Intents with confidence below 0.3 are
   discarded rather than dispatched. Ambiguous intents (0.3-0.6) trigger a
   clarification turn rather than silent failure.

8. **Response mode consistency**: A response that began in `VoiceOnly` mode
   completes in `VoiceOnly` mode even if mode changes mid-response. Mode
   changes take effect on the next turn.

## Voice Pipeline Flow

```
Microphone input
    |
    v
+------------------------+
| VoiceActivityDetector  |  3-second rolling buffer
| (always listening)     |  adaptive threshold
+------------------------+
    |  speech detected
    v
+------------------------+
| Wake Word Detection    |  on-device model
+------------------------+
    |  confirmed
    v
+------------------------+
| SpeechToText (STT)     |  on-device Whisper variant
| streaming partials     |  final transcript
+------------------------+
    |
    v
+------------------------+
| SpeakerContext extract  |  embedding, emotion, urgency, noise
+------------------------+
    |
    v
+------------------------+
| MultiIntentDecomposer  |  1 utterance -> N intents
| + IntentClassifier     |  domain + action + entities
+------------------------+
    |                          |
    | intents                  | speaker context (normalized)
    v                          v
+------------------------+  +------------------------+
| Kernel Syscall dispatch|  | TinyDancerRouter       |
| (DDD-002)              |  | 18-dim with audio feats|
+------------------------+  +------------------------+
    |
    v
+------------------------+
| Intent results         |
+------------------------+
    |
    v
+------------------------+
| TTS Synthesis          |  domain persona selection
| Opus streaming         |  <200ms to first chunk
+------------------------+
    |
    v
+------------------------+
| MultimodalResponse     |  voice + visual + haptic
| delivery               |
+------------------------+
```

## Context Relationships

| Related Context | Relationship | Description |
|----------------|-------------|-------------|
| Kernel Syscall (DDD-002) | Upstream | Voice dispatches `VoiceTranscribe`, `VoiceSynthesize`, `IntentRoute` syscalls to the kernel |
| Inference Routing (DDD-005) | Upstream | `TinyDancerRouter` (18-dim input) routes voice queries using audio-derived features (emotion, urgency, noise) |
| Swarm Coordination (DDD-004) | Upstream | Multi-intent fan-out uses `Strategy::Swarm` for parallel intent resolution across zones |
| Agent Lifecycle (DDD-003) | Downstream | Agents receive structured intents; voice context does not know agent internals |
| Phone Runtime (DDD-009) | Partner | Voice pipeline runs within phone runtime, shares battery/thermal state; ResponseMode is informed by phone sensors |
| Research Evolution (DDD-006) | Downstream | Voice session patterns feed into research evolution for intent classification improvement |

### Relationship Details

**Upstream to Kernel**: Voice translates audio into `Intent` structs and
dispatches them as syscalls. The kernel treats voice intents identically to
text-originated intents -- the voice context is responsible for ensuring the
translation is complete before dispatch.

**Partner with Phone Runtime**: The voice pipeline and phone runtime share
hardware resources (microphone, speaker, battery). The phone runtime provides
sensor state that determines `ResponseMode`. Neither context owns the other;
they coordinate through shared events on the `DomainEventBus`.

**Fan-out via Swarm**: When a single utterance decomposes into multiple intents
across different life domains, the voice context requests scatter-gather
execution via `Strategy::Swarm { scatter_zones, gather_strategy, timeout_ms }`.
Results are assembled into a single `MultimodalResponse`.

## Anti-Corruption Layer

The voice context maintains strict boundaries to protect both itself and
downstream consumers:

- **Audio isolation**: Raw audio never crosses the context boundary. The ACL
  translates audio into text transcripts and scalar features (confidence,
  duration, noise level). The kernel and all downstream contexts operate on
  text and structured data only.

- **Speaker context normalization**: `SpeakerContext` fields are normalized
  to `[0, 1]` floats (or `[-1, 1]` for emotion valence) before passing to
  the router. The router receives a fixed-size numeric vector, not raw audio
  features.

- **Emotion bucketing**: Per ADR-017, continuous emotion valence (`f32`) is
  quantized to 5 discrete buckets at the ACL boundary. This prevents
  fingerprinting through high-precision emotion tracking.

- **TTS encapsulation**: TTS output is Opus-encoded chunks. Downstream
  consumers receive `OpusChunk` structs and do not need to understand
  synthesis model internals, voice cloning parameters, or SSML markup.

- **Session memory via SONA**: Session summaries are stored through the SONA
  `PatternBank` API (`rlmx-cognitive`), not through direct database access.
  This ensures voice session data follows the same consolidation and privacy
  policies as all other learned patterns.

- **LifeDomain re-export**: The `LifeDomain` enum is defined in `rlmx-kernel`
  and re-exported by `rlmx-voice`. The voice context never defines its own
  domain taxonomy -- it consumes the canonical kernel definition.

## File Map

| File | Types | Status |
|------|-------|--------|
| `crates/rlmx-voice/src/lib.rs` | `VoiceSession`, `VoicePipeline`, `VoiceDomainEvent` | Planned |
| `crates/rlmx-voice/src/vad.rs` | `VoiceActivityDetector`, wake word detection | Planned |
| `crates/rlmx-voice/src/stt.rs` | `SpeechToText`, streaming transcription | Planned |
| `crates/rlmx-voice/src/intent.rs` | `Intent`, `IntentClassifier`, `MultiIntentDecomposer` | Planned |
| `crates/rlmx-voice/src/tts.rs` | `TextToSpeech`, `VoicePersona`, Opus streaming | Planned |
| `crates/rlmx-voice/src/session.rs` | `VoiceSession`, `ConversationTurn`, session lifecycle | Planned |
| `crates/rlmx-voice/src/response.rs` | `MultimodalResponse`, `ResponseMode`, response assembly | Planned |
| `crates/rlmx-voice/src/speaker.rs` | `SpeakerContext`, on-device speaker identification | Planned |
| `crates/rlmx-voice/src/acl.rs` | Anti-corruption layer, emotion bucketing, normalization | Planned |

## Implementation Status

**Status**: Implemented in `crates/rlmx-voice/`

| Module | File | Tests |
|--------|------|-------|
| VoicePipeline | `src/pipeline.rs` | 13 |
| VoiceSession (aggregate root) | `src/session.rs` | 14 |
| MultiIntentDecomposer | `src/intent.rs` | 16 |
| VoiceActivityDetector | `src/vad.rs` | 11 |
| TtsEngine | `src/tts.rs` | 12 |

**Total**: 66 tests, all passing.

All entities, value objects, and domain events from DDD-008 are implemented.
VoiceSession is the aggregate root owning conversation turns, emotion trajectory, and intent state.
Cross-context communication uses domain events (VoiceDomainEvent with 8 variants).
Privacy invariants enforced: no audio stored, 3s rolling buffer, on-device only.
