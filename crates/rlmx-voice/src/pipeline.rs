//! Voice pipeline orchestrating the full voice flow:
//! VAD -> STT -> Intent Classification -> Multi-Intent Decomposition -> TTS.
//!
//! The pipeline is an entity (not a value object) because it has independent
//! lifecycle and mutable state in its VAD calibration and STT adaptation.

use crate::intent::MultiIntentDecomposer;
use crate::session::{SessionMemory, VoiceDomainEvent, VoiceSession};
use crate::tts::{TtsChunk, TtsEngine};
use crate::vad::{AudioFrame, VadDecision, VoiceActivityDetector};
use crate::{VoiceError, VoiceResult};
use rlmx_kernel::{Intent, LifeDomain, ResponseMode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// STT inference tier for tiered engine escalation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SttTier {
    /// On-device Whisper-tiny Q4 (~95% accuracy).
    Small,
    /// Laptop/tablet Whisper-base (~97% accuracy).
    Medium,
    /// Cloud Whisper-large (~99% accuracy).
    Remote,
}

/// Stub speech-to-text engine with tiered escalation.
///
/// Uses TieredEngine concept from ADR-004: Small (phone) -> Medium (laptop)
/// -> Remote (cloud). Escalates when confidence is below threshold (0.4).
pub struct SpeechToText {
    /// Current inference tier.
    pub tier: SttTier,
    /// Confidence threshold for escalation.
    pub escalation_threshold: f32,
}

impl Default for SpeechToText {
    fn default() -> Self {
        Self {
            tier: SttTier::Small,
            escalation_threshold: 0.4,
        }
    }
}

impl SpeechToText {
    /// Create a new STT engine at a specific tier.
    pub fn new(tier: SttTier) -> Self {
        Self {
            tier,
            escalation_threshold: 0.4,
        }
    }

    /// Transcribe audio frames into text (stub).
    ///
    /// Returns (transcript, confidence, tier_used). If confidence at the
    /// current tier is below the escalation threshold, escalates to the
    /// next tier (if available).
    pub fn transcribe(&self, _frames: &[AudioFrame]) -> SttResult {
        // Stub: simulate transcription with tier-dependent confidence.
        let (confidence, tier_used) = match self.tier {
            SttTier::Small => (0.85, SttTier::Small),
            SttTier::Medium => (0.92, SttTier::Medium),
            SttTier::Remote => (0.98, SttTier::Remote),
        };

        SttResult {
            transcript: String::new(), // Stub: caller provides transcript.
            confidence,
            tier_used,
            latency_ms: match tier_used {
                SttTier::Small => 45,
                SttTier::Medium => 80,
                SttTier::Remote => 200,
            },
            is_final: true,
        }
    }

    /// Simulate transcription of a known text (for testing).
    pub fn transcribe_text(&self, text: &str) -> SttResult {
        let base_confidence = match self.tier {
            SttTier::Small => 0.85,
            SttTier::Medium => 0.92,
            SttTier::Remote => 0.98,
        };

        // Simulate escalation if confidence is below threshold.
        let (confidence, tier_used) = if base_confidence < self.escalation_threshold {
            // Escalate.
            match self.tier {
                SttTier::Small => (0.92, SttTier::Medium),
                SttTier::Medium => (0.98, SttTier::Remote),
                SttTier::Remote => (0.98, SttTier::Remote),
            }
        } else {
            (base_confidence, self.tier)
        };

        SttResult {
            transcript: text.to_string(),
            confidence,
            tier_used,
            latency_ms: match tier_used {
                SttTier::Small => 45,
                SttTier::Medium => 80,
                SttTier::Remote => 200,
            },
            is_final: true,
        }
    }
}

/// Result of speech-to-text processing.
#[derive(Debug, Clone)]
pub struct SttResult {
    /// The transcribed text.
    pub transcript: String,
    /// Confidence score in [0, 1].
    pub confidence: f32,
    /// Which tier actually performed the transcription.
    pub tier_used: SttTier,
    /// Latency in milliseconds.
    pub latency_ms: u64,
    /// Whether this is the final transcript (vs. a partial).
    pub is_final: bool,
}

/// The voice pipeline orchestrating VAD -> STT -> Intent -> TTS.
///
/// Initialized at app start, shared across sessions. Has mutable state
/// in VAD threshold calibration and session memory.
pub struct VoicePipeline {
    /// Voice activity detector (always listening).
    pub vad: VoiceActivityDetector,
    /// Speech-to-text engine with tiered escalation.
    pub stt: SpeechToText,
    /// Multi-intent decomposer.
    pub decomposer: MultiIntentDecomposer,
    /// Session memory for long-term storage.
    pub session_memory: SessionMemory,
    /// Intent confidence threshold for dispatch.
    ///
    /// Intents below this confidence are filtered by the decomposer.
    /// Exposed via `intent_confidence_threshold()` accessor.
    _intent_confidence_threshold: f32,
}

impl VoicePipeline {
    /// Create a new voice pipeline with default settings.
    pub fn new() -> Self {
        Self {
            vad: VoiceActivityDetector::default(),
            stt: SpeechToText::default(),
            decomposer: MultiIntentDecomposer::new(0.3),
            session_memory: SessionMemory::new(),
            _intent_confidence_threshold: 0.3,
        }
    }

    /// Create a pipeline with custom VAD threshold and STT tier.
    pub fn with_config(vad_threshold: f32, stt_tier: SttTier) -> Self {
        Self {
            vad: VoiceActivityDetector::new(vad_threshold),
            stt: SpeechToText::new(stt_tier),
            decomposer: MultiIntentDecomposer::new(0.3),
            session_memory: SessionMemory::new(),
            _intent_confidence_threshold: 0.3,
        }
    }

    /// Start a new voice session.
    pub fn start_session(&self, mode: ResponseMode) -> (VoiceSession, VoiceDomainEvent) {
        let session = VoiceSession::new(mode);
        let event = VoiceDomainEvent::SessionStarted {
            session_id: session.id,
            speaker_id: session.speaker_id,
            mode,
        };
        tracing::info!(session_id = %session.id, ?mode, "voice session started");
        (session, event)
    }

    /// Process audio frames through VAD.
    ///
    /// Returns the VAD decision. When speech is detected (`SpeechStart` or
    /// `SpeechContinue`), callers should emit a `SpeechDetected` event via
    /// [`detect_speech_with_event`] if a session is active.
    pub fn detect_speech(&self, frames: &[AudioFrame]) -> VadDecision {
        self.vad.process_buffer(frames)
    }

    /// Process audio frames through VAD and emit a domain event when speech
    /// is detected (DDD-008 `SpeechDetected` event).
    pub fn detect_speech_with_event(
        &self,
        session_id: Uuid,
        frames: &[AudioFrame],
    ) -> (VadDecision, Option<VoiceDomainEvent>) {
        let decision = self.vad.process_buffer(frames);
        let event = match decision {
            VadDecision::SpeechStart | VadDecision::SpeechContinue => {
                // Compute a rough confidence from the average energy of the frames.
                let avg_energy = if frames.is_empty() {
                    0.0
                } else {
                    frames.iter().map(|f| f.energy).sum::<f32>() / frames.len() as f32
                };
                Some(VoiceDomainEvent::SpeechDetected {
                    session_id,
                    vad_confidence: avg_energy.clamp(0.0, 1.0),
                })
            }
            _ => None,
        };
        (decision, event)
    }

    /// Process a transcript through intent decomposition.
    ///
    /// Returns the intents and a domain event. Intents below the confidence
    /// threshold (0.3) are already filtered by the decomposer.
    pub fn process_transcript(
        &self,
        session: &mut VoiceSession,
        transcript: &str,
    ) -> VoiceResult<(Vec<Intent>, Vec<VoiceDomainEvent>)> {
        if transcript.trim().is_empty() {
            return Err(VoiceError::Stt("empty transcript".to_string()));
        }

        let mut events = Vec::new();

        // STT result event.
        let stt_result = self.stt.transcribe_text(transcript);
        events.push(VoiceDomainEvent::TranscriptReady {
            session_id: session.id,
            transcript: stt_result.transcript.clone(),
            stt_confidence: stt_result.confidence,
            is_final: stt_result.is_final,
        });

        // Decompose intents.
        let intents = self.decomposer.decompose(transcript);

        if !intents.is_empty() {
            events.push(VoiceDomainEvent::IntentsDecomposed {
                session_id: session.id,
                intents: intents.clone(),
            });
        }

        // Update session.
        session.add_user_turn(transcript.to_string(), intents.clone());

        tracing::debug!(
            session_id = %session.id,
            intent_count = intents.len(),
            "transcript processed"
        );

        Ok((intents, events))
    }

    /// Synthesize a response for the primary intent's domain.
    pub fn synthesize_response(
        &self,
        session: &mut VoiceSession,
        response_text: &str,
        domain: LifeDomain,
    ) -> (Vec<TtsChunk>, VoiceDomainEvent) {
        let engine = TtsEngine::for_domain(domain);
        let chunks = engine.synthesize(response_text);

        session.add_system_turn(response_text.to_string());

        let event = VoiceDomainEvent::ResponseStreaming {
            session_id: session.id,
            domain: format!("{:?}", domain),
            chunk_index: 0,
        };

        (chunks, event)
    }

    /// End a session and store its summary.
    pub fn end_session(&mut self, session: &VoiceSession) -> VoiceDomainEvent {
        self.session_memory.store(session);

        let event = VoiceDomainEvent::SessionEnded {
            session_id: session.id,
            turn_count: session.turn_count() as u32,
            duration_ms: session.duration_ms(),
        };

        tracing::info!(
            session_id = %session.id,
            turns = session.turn_count(),
            "voice session ended"
        );

        event
    }

    /// Get the session memory for querying past sessions.
    pub fn session_memory(&self) -> &SessionMemory {
        &self.session_memory
    }
}

impl Default for VoicePipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Full pipeline processing result.
#[derive(Debug)]
pub struct PipelineResult {
    /// Session ID.
    pub session_id: Uuid,
    /// Extracted intents.
    pub intents: Vec<Intent>,
    /// TTS chunks for the response.
    pub response_chunks: Vec<TtsChunk>,
    /// All domain events emitted during processing.
    pub events: Vec<VoiceDomainEvent>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vad::AudioFrame;

    fn speech_frames(count: usize) -> Vec<AudioFrame> {
        (0..count)
            .map(|_| AudioFrame {
                energy: 0.6,
                spectral_centroid: 0.5,
                duration_ms: 20,
            })
            .collect()
    }

    fn silence_frames(count: usize) -> Vec<AudioFrame> {
        (0..count)
            .map(|_| AudioFrame {
                energy: 0.02,
                spectral_centroid: 0.1,
                duration_ms: 20,
            })
            .collect()
    }

    #[test]
    fn test_pipeline_creation() {
        let pipeline = VoicePipeline::new();
        assert_eq!(pipeline.stt.tier, SttTier::Small);
        assert_eq!(pipeline.session_memory.count(), 0);
    }

    #[test]
    fn test_pipeline_with_config() {
        let pipeline = VoicePipeline::with_config(0.2, SttTier::Medium);
        assert_eq!(pipeline.stt.tier, SttTier::Medium);
    }

    #[test]
    fn test_pipeline_start_session() {
        let pipeline = VoicePipeline::new();
        let (session, event) = pipeline.start_session(ResponseMode::Multimodal);
        assert_eq!(session.mode, ResponseMode::Multimodal);
        assert!(matches!(event, VoiceDomainEvent::SessionStarted { .. }));
    }

    #[test]
    fn test_pipeline_detect_speech() {
        let pipeline = VoicePipeline::new();
        let decision = pipeline.detect_speech(&speech_frames(5));
        assert_eq!(decision, VadDecision::SpeechContinue);
    }

    #[test]
    fn test_pipeline_detect_silence() {
        let pipeline = VoicePipeline::new();
        let decision = pipeline.detect_speech(&silence_frames(5));
        assert_eq!(decision, VadDecision::Silence);
    }

    #[test]
    fn test_pipeline_process_transcript() {
        let pipeline = VoicePipeline::new();
        let (mut session, _) = pipeline.start_session(ResponseMode::Multimodal);
        let result = pipeline.process_transcript(&mut session, "pay my electricity bill");
        assert!(result.is_ok());
        let (intents, events) = result.unwrap();
        assert!(!intents.is_empty());
        assert!(!events.is_empty());
        assert_eq!(session.turn_count(), 1);
    }

    #[test]
    fn test_pipeline_process_empty_transcript_error() {
        let pipeline = VoicePipeline::new();
        let (mut session, _) = pipeline.start_session(ResponseMode::VoiceOnly);
        let result = pipeline.process_transcript(&mut session, "");
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_synthesize_response() {
        let pipeline = VoicePipeline::new();
        let (mut session, _) = pipeline.start_session(ResponseMode::Multimodal);
        let (chunks, event) = pipeline.synthesize_response(
            &mut session,
            "Your bill has been paid.",
            LifeDomain::Finance,
        );
        assert!(!chunks.is_empty());
        assert!(matches!(event, VoiceDomainEvent::ResponseStreaming { .. }));
        assert_eq!(session.turn_count(), 1);
    }

    #[test]
    fn test_pipeline_end_session() {
        let mut pipeline = VoicePipeline::new();
        let (session, _) = pipeline.start_session(ResponseMode::Multimodal);
        let event = pipeline.end_session(&session);
        assert!(matches!(event, VoiceDomainEvent::SessionEnded { .. }));
        assert_eq!(pipeline.session_memory().count(), 1);
    }

    #[test]
    fn test_pipeline_full_flow() {
        let mut pipeline = VoicePipeline::new();

        // 1. Start session.
        let (mut session, _start_event) = pipeline.start_session(ResponseMode::Multimodal);

        // 2. Detect speech.
        let decision = pipeline.detect_speech(&speech_frames(5));
        assert_eq!(decision, VadDecision::SpeechContinue);

        // 3. Process transcript.
        let (intents, _events) = pipeline
            .process_transcript(&mut session, "order some groceries")
            .unwrap();
        assert!(!intents.is_empty());

        // 4. Synthesize response.
        let (chunks, _) = pipeline.synthesize_response(
            &mut session,
            "I will order groceries for you.",
            LifeDomain::Shopping,
        );
        assert!(!chunks.is_empty());

        // 5. End session.
        let end_event = pipeline.end_session(&session);
        assert!(matches!(end_event, VoiceDomainEvent::SessionEnded { .. }));
        assert_eq!(session.turn_count(), 2); // user + system
    }

    #[test]
    fn test_stt_tier_escalation() {
        let stt = SpeechToText::new(SttTier::Small);
        let result = stt.transcribe_text("test input");
        assert_eq!(result.tier_used, SttTier::Small);
        assert!(result.confidence >= 0.8);
    }

    #[test]
    fn test_stt_all_tiers() {
        for tier in [SttTier::Small, SttTier::Medium, SttTier::Remote] {
            let stt = SpeechToText::new(tier);
            let result = stt.transcribe_text("hello");
            assert_eq!(result.tier_used, tier);
            assert!(result.is_final);
        }
    }

    #[test]
    fn test_pipeline_multi_intent_flow() {
        let pipeline = VoicePipeline::new();
        let (mut session, _) = pipeline.start_session(ResponseMode::Multimodal);

        let (intents, events) = pipeline
            .process_transcript(
                &mut session,
                "cancel my dentist appointment and order more dog food",
            )
            .unwrap();

        // Should have at least 2 intents.
        assert!(
            intents.len() >= 2,
            "expected >= 2 intents, got {}",
            intents.len()
        );
        // Should have TranscriptReady and IntentsDecomposed events.
        assert!(events.len() >= 2);
    }

    #[test]
    fn test_pipeline_default_trait() {
        let pipeline = VoicePipeline::default();
        assert_eq!(pipeline.stt.tier, SttTier::Small);
    }

    #[test]
    fn test_detect_speech_with_event_emits_on_speech() {
        let pipeline = VoicePipeline::new();
        let session_id = Uuid::new_v4();
        let (decision, event) = pipeline.detect_speech_with_event(session_id, &speech_frames(5));
        assert_eq!(decision, VadDecision::SpeechContinue);
        assert!(event.is_some());
        match event.unwrap() {
            VoiceDomainEvent::SpeechDetected {
                session_id: sid,
                vad_confidence,
            } => {
                assert_eq!(sid, session_id);
                assert!(vad_confidence > 0.0);
            }
            _ => panic!("expected SpeechDetected event"),
        }
    }

    #[test]
    fn test_detect_speech_with_event_none_on_silence() {
        let pipeline = VoicePipeline::new();
        let session_id = Uuid::new_v4();
        let (decision, event) = pipeline.detect_speech_with_event(session_id, &silence_frames(5));
        assert_eq!(decision, VadDecision::Silence);
        assert!(event.is_none());
    }
}
