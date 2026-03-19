# ADR-012: Voice-First Pipeline Architecture

| Field    | Value                     |
|----------|---------------------------|
| Status   | Implemented               |
| Date     | 2026-03-19                |
| Authors  | Cedric                    |
| Replaces | —                         |

## Context

RLMX is a cognition kernel for LLM agents. The PRD mandates a voice-first consumer superapp where users speak into their phone mic and trigger millions of distributed agents. Currently there is no voice capability — no STT, TTS, VAD, or voice-aware routing.

The voice pipeline must:
- Run entirely on-device for privacy (no audio leaves the device)
- Use Whisper-tiny Q4 for STT via existing `rlmx-ruvllm` TieredEngine
- Include always-listening VAD (500K param CNN from `ruv-neural-signal`)
- Support user-configurable wake words (on-device keyword spotting, 20K params)
- Enable multi-intent decomposition from a single utterance

## Decision

### Introduce `rlmx-voice` crate

New crate `crates/rlmx-voice/` with these components:
- `VoicePipeline` struct: orchestrates VAD → STT → Intent → Decomposer → TTS
- `VoiceActivityDetector`: 500K param CNN, energy-based + spectral features, <0.1% CPU background
- `SpeechToText`: Whisper-tiny Q4 GGUF on-device, <100ms latency, with TieredEngine escalation (phone → laptop → cloud)
- `IntentClassifier`: delegates to expanded TinyDancerRouter (18→32→5, see ADR-019)
- `MultiIntentDecomposer`: parses single utterance into multiple Intent structs routed to different agent domains
- `TextToSpeech`: on-device for <50 tokens, cloud streaming for long-form, persona system (6 domain-specific voices)
- `SessionMemory`: SONA PatternBank-backed voice session storage with temporal-weighted VecSearch
- `VoiceSession`: tracks speaker embedding, conversation turns, active intents, response mode (VoiceOnly/Visual/Multimodal/Ambient)

### Privacy invariant

- Audio processed in 3-second rolling buffer, never stored
- VAD outputs binary "speech detected" signal — only then does STT activate
- Zero audio transmitted until wake word detected
- Speaker identification via on-device voice embeddings only

### Integration with existing kernel

- STT uses `TieredEngine` escalation path: Small (phone Whisper-tiny) → Medium (laptop Whisper-base) → Remote (cloud Whisper-large)
- Intent classification reuses `TinyDancerRouter` with 4 additional input dimensions (speaker_confidence, emotion_valence, urgency_score, ambient_noise_level)
- Multi-intent results dispatched via `Strategy::Swarm` with `GatherStrategy::All`
- Voice sessions stored via `SyscallPermission::StateMutate` through existing kernel

### TTS Persona System

6 domain-specific voice personas:

| Domain    | Style                  |
|-----------|------------------------|
| Finance   | Calm, precise          |
| Health    | Empathetic, warm       |
| Legal     | Direct, authoritative  |
| Shopping  | Upbeat on savings      |
| Calendar  | Brief, action-oriented |
| Emergency | Urgent, clear          |

Streaming: first sentence plays within 200ms of generation start.

## Consequences

### Positive

- Voice-first interaction without any cloud dependency for basic usage
- Privacy by design — no audio leaves device
- Reuses existing TieredEngine, TinyDancerRouter, and SONA infrastructure
- Multi-intent decomposition enables "one utterance → many agents" pattern

### Negative

- Whisper-tiny Q4 accuracy (~95%) may frustrate users with accents or in noisy environments → mitigated by TieredEngine escalation
- Always-listening VAD adds ~2% battery drain/day → mitigated by hardware DSP on modern SoCs
- New crate adds build complexity

### Risks

- On-device TTS quality may not match cloud services initially
- Wake word false positives in noisy households
- Language support limited by on-device model size (99 languages at varying quality)

## References

- ADR-004: Tiered Model Inference (TieredEngine escalation)
- ADR-003: Neural Model Routing (TinyDancerRouter)
- ADR-019: Kernel Expansion (new syscalls and router dims)
- PRD Section 3: Voice-First Architecture

## Implementation Notes

Implemented in `crates/rlmx-voice/` with 6 source files and 66 tests:
- `pipeline.rs`: VoicePipeline orchestrating VAD -> STT -> Intent -> TTS
- `vad.rs`: VoiceActivityDetector with energy threshold + spectral features, 3s rolling buffer
- `intent.rs`: IntentClassifier + MultiIntentDecomposer for 12 life domains
- `tts.rs`: TtsEngine with 6 domain personas (Finance/Health/Legal/Shopping/Calendar/Emergency)
- `session.rs`: VoiceSession aggregate root with conversation turns, emotion trajectory, session memory
- CLI: `cargo run -p rlmx-cli -- voice start|transcribe|intents|session`
- Hooks: `scripts/hooks/pre-voice.sh`, `scripts/hooks/post-voice.sh`
