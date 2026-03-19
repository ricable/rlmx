//! # rlmx-voice
//!
//! Voice-first pipeline for the RLMX cognition kernel (ADR-012, DDD-008).
//!
//! Provides always-listening VAD, on-device STT with tiered escalation,
//! multi-intent decomposition across 12 life domains, persona-based TTS,
//! and voice session management with temporal-weighted memory.
//!
//! Privacy invariant: audio is processed in a 3-second rolling buffer and
//! never persisted. Only text transcripts and scalar features cross the
//! context boundary.

pub mod intent;
pub mod pipeline;
pub mod session;
pub mod tts;
pub mod vad;

// Re-export primary types for convenient access.
pub use intent::{
    ExtractedEntity, Intent, IntentClassifier, LifeDomain, MultiIntentDecomposer,
};
pub use pipeline::{SpeechToText, SttTier, VoicePipeline};
pub use session::{
    ConversationTurn, ResponseMode, SessionMemory, TurnDirection, VoiceDomainEvent, VoiceSession,
};
pub use tts::{PersonaStyle, TtsEngine, VoicePersona};
pub use vad::VoiceActivityDetector;

/// Errors specific to the voice pipeline.
#[derive(Debug, thiserror::Error)]
pub enum VoiceError {
    #[error("VAD error: {0}")]
    Vad(String),

    #[error("STT error: {0}")]
    Stt(String),

    #[error("intent classification error: {0}")]
    Intent(String),

    #[error("TTS error: {0}")]
    Tts(String),

    #[error("session error: {0}")]
    Session(String),

    #[error("pipeline not initialized")]
    NotInitialized,

    #[error("session timed out after {0}ms")]
    SessionTimeout(u64),

    #[error("confidence below threshold: {confidence:.3} < {threshold:.3}")]
    LowConfidence { confidence: f32, threshold: f32 },
}

pub type VoiceResult<T> = Result<T, VoiceError>;
