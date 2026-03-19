//! Text-to-speech with 6+ persona variants and streaming support.
//!
//! Each life domain has a default voice persona with appropriate style,
//! speech rate, and model. Streaming TTS targets <200ms to first audio
//! chunk (stub implementation).

use crate::intent::LifeDomain;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Voice persona style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersonaStyle {
    /// Empathetic, gentle tone.
    Warm,
    /// Confident, direct tone.
    Authoritative,
    /// Enthusiastic, positive tone.
    Upbeat,
    /// Measured, precise tone.
    Calm,
    /// Default balanced tone.
    Neutral,
    /// Fast, clear, high-priority tone.
    Urgent,
}

/// A voice persona configuration for TTS output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicePersona {
    /// Which life domain this persona serves.
    pub domain: LifeDomain,
    /// The persona's vocal style.
    pub style: PersonaStyle,
    /// Speech rate multiplier (0.5 = half speed, 2.0 = double speed).
    pub rate: f32,
    /// TTS model identifier.
    pub model_id: String,
}

impl VoicePersona {
    /// Get the default persona for a given life domain.
    ///
    /// Maps domains to styles per ADR-012 persona table:
    /// Finance=Calm, Health=Warm, Legal=Authoritative, Shopping=Upbeat,
    /// Calendar=Neutral (brief), Emergency=Urgent.
    pub fn for_domain(domain: LifeDomain) -> Self {
        let (style, rate, model) = match domain {
            LifeDomain::Finance => (PersonaStyle::Calm, 1.0, "ruvix-tts-calm-v1"),
            LifeDomain::Health => (PersonaStyle::Warm, 0.95, "ruvix-tts-warm-v1"),
            LifeDomain::Legal => (PersonaStyle::Authoritative, 1.0, "ruvix-tts-auth-v1"),
            LifeDomain::Shopping => (PersonaStyle::Upbeat, 1.1, "ruvix-tts-upbeat-v1"),
            LifeDomain::Calendar => (PersonaStyle::Neutral, 1.15, "ruvix-tts-neutral-v1"),
            LifeDomain::Emergency => (PersonaStyle::Urgent, 1.2, "ruvix-tts-urgent-v1"),
            LifeDomain::Travel => (PersonaStyle::Upbeat, 1.05, "ruvix-tts-upbeat-v1"),
            LifeDomain::Education => (PersonaStyle::Warm, 0.9, "ruvix-tts-warm-v1"),
            LifeDomain::Entertainment => (PersonaStyle::Upbeat, 1.1, "ruvix-tts-upbeat-v1"),
            LifeDomain::HomeAutomation => (PersonaStyle::Neutral, 1.0, "ruvix-tts-neutral-v1"),
            LifeDomain::Communication => (PersonaStyle::Neutral, 1.0, "ruvix-tts-neutral-v1"),
            LifeDomain::Productivity => (PersonaStyle::Calm, 1.05, "ruvix-tts-calm-v1"),
        };
        Self {
            domain,
            style,
            rate,
            model_id: model.to_string(),
        }
    }
}

/// A single Opus-encoded audio chunk for streaming TTS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsChunk {
    /// Chunk index within the stream.
    pub index: u32,
    /// Encoded audio data (stub: empty in this implementation).
    pub data: Vec<u8>,
    /// Duration of this chunk in milliseconds.
    pub duration_ms: u16,
}

/// Stub TTS engine with streaming support.
///
/// In production, this would use on-device neural TTS for short responses
/// (<50 tokens) and cloud streaming for long-form content.
pub struct TtsEngine {
    /// The active voice persona.
    persona: VoicePersona,
    /// Target chunk duration in milliseconds.
    chunk_duration_ms: u16,
}

impl TtsEngine {
    /// Create a new TTS engine with the given persona.
    pub fn new(persona: VoicePersona) -> Self {
        Self {
            persona,
            chunk_duration_ms: 80,
        }
    }

    /// Create a TTS engine for a specific life domain using its default persona.
    pub fn for_domain(domain: LifeDomain) -> Self {
        Self::new(VoicePersona::for_domain(domain))
    }

    /// Get the current persona.
    pub fn persona(&self) -> &VoicePersona {
        &self.persona
    }

    /// Set a new persona.
    pub fn set_persona(&mut self, persona: VoicePersona) {
        self.persona = persona;
    }

    /// Synthesize text into a sequence of TTS chunks (stub).
    ///
    /// Returns simulated Opus chunks. Each chunk represents ~80ms of audio.
    /// The number of chunks scales with text length and speech rate.
    pub fn synthesize(&self, text: &str) -> Vec<TtsChunk> {
        if text.is_empty() {
            return Vec::new();
        }

        // Estimate audio duration: ~150ms per word at 1.0x rate.
        let word_count = text.split_whitespace().count().max(1);
        let total_duration_ms = (word_count as f32 * 150.0 / self.persona.rate) as u32;
        let chunk_count = (total_duration_ms / self.chunk_duration_ms as u32).max(1);

        (0..chunk_count)
            .map(|i| TtsChunk {
                index: i,
                data: vec![0u8; 160], // Stub: 160 bytes per chunk placeholder.
                duration_ms: self.chunk_duration_ms,
            })
            .collect()
    }

    /// Estimate the latency to first chunk in milliseconds.
    ///
    /// Stub: returns a simulated value. Target is <200ms per ADR-012.
    pub fn estimated_first_chunk_latency_ms(&self) -> u64 {
        // On-device: ~50ms model inference + ~20ms encoding.
        70
    }

    /// Get the stream ID for tracking this synthesis operation.
    pub fn stream_id() -> Uuid {
        Uuid::new_v4()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_finance() {
        let p = VoicePersona::for_domain(LifeDomain::Finance);
        assert_eq!(p.style, PersonaStyle::Calm);
        assert!((p.rate - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_persona_emergency() {
        let p = VoicePersona::for_domain(LifeDomain::Emergency);
        assert_eq!(p.style, PersonaStyle::Urgent);
        assert!(p.rate > 1.0);
    }

    #[test]
    fn test_persona_health() {
        let p = VoicePersona::for_domain(LifeDomain::Health);
        assert_eq!(p.style, PersonaStyle::Warm);
    }

    #[test]
    fn test_persona_shopping() {
        let p = VoicePersona::for_domain(LifeDomain::Shopping);
        assert_eq!(p.style, PersonaStyle::Upbeat);
    }

    #[test]
    fn test_persona_legal() {
        let p = VoicePersona::for_domain(LifeDomain::Legal);
        assert_eq!(p.style, PersonaStyle::Authoritative);
    }

    #[test]
    fn test_persona_calendar() {
        let p = VoicePersona::for_domain(LifeDomain::Calendar);
        assert_eq!(p.style, PersonaStyle::Neutral);
    }

    #[test]
    fn test_all_12_domains_have_personas() {
        for domain in LifeDomain::all() {
            let p = VoicePersona::for_domain(*domain);
            assert_eq!(p.domain, *domain);
            assert!(p.rate > 0.0);
            assert!(!p.model_id.is_empty());
        }
    }

    #[test]
    fn test_synthesize_produces_chunks() {
        let engine = TtsEngine::for_domain(LifeDomain::Finance);
        let chunks = engine.synthesize("Hello, this is a test of the TTS engine.");
        assert!(!chunks.is_empty());
        assert_eq!(chunks[0].index, 0);
        assert!(chunks.last().unwrap().index == chunks.len() as u32 - 1);
    }

    #[test]
    fn test_synthesize_empty_text() {
        let engine = TtsEngine::for_domain(LifeDomain::Health);
        let chunks = engine.synthesize("");
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_first_chunk_latency_under_200ms() {
        let engine = TtsEngine::for_domain(LifeDomain::Emergency);
        assert!(engine.estimated_first_chunk_latency_ms() < 200);
    }

    #[test]
    fn test_faster_rate_fewer_chunks() {
        let slow = TtsEngine::new(VoicePersona {
            domain: LifeDomain::Finance,
            style: PersonaStyle::Calm,
            rate: 0.5,
            model_id: "test".to_string(),
        });
        let fast = TtsEngine::new(VoicePersona {
            domain: LifeDomain::Finance,
            style: PersonaStyle::Calm,
            rate: 2.0,
            model_id: "test".to_string(),
        });
        let text = "This is a sentence with several words for testing.";
        assert!(slow.synthesize(text).len() > fast.synthesize(text).len());
    }
}
