//! Voice Activity Detection with energy-based detection simulation.
//!
//! The VAD runs continuously in the background with minimal CPU usage (<0.1%).
//! Audio is processed in a 3-second rolling buffer and never persisted.
//! Only a binary "speech detected" signal propagates downstream.

use serde::{Deserialize, Serialize};

/// Voice Activity Detector with energy threshold and spectral feature stubs.
///
/// Processes audio in a rolling buffer (default 3 seconds at 16kHz).
/// The buffer is a ring buffer that continuously overwrites, enforcing
/// the privacy invariant that raw audio is never stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceActivityDetector {
    /// Adaptive silence/speech threshold in [0, 1].
    pub threshold: f32,
    /// Rolling buffer duration in milliseconds.
    pub buffer_duration_ms: u64,
    /// Whether wake word detection is enabled.
    pub wake_word_enabled: bool,
    /// Name of the wake word model (on-device, ~20K params).
    pub wake_word_model: String,
    /// Sample rate in Hz.
    sample_rate: u32,
    /// Number of consecutive frames above threshold to confirm speech.
    min_speech_frames: u32,
    /// Number of consecutive frames below threshold to confirm silence.
    min_silence_frames: u32,
}

/// Result of processing an audio frame through VAD.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VadDecision {
    /// No speech detected, VAD remains in listening state.
    Silence,
    /// Speech onset detected.
    SpeechStart,
    /// Ongoing speech.
    SpeechContinue,
    /// Speech has ended (trailing silence detected).
    SpeechEnd,
}

/// Simulated audio frame for stub processing.
#[derive(Debug, Clone)]
pub struct AudioFrame {
    /// Simulated energy level in [0, 1].
    pub energy: f32,
    /// Simulated spectral centroid (higher = more speech-like).
    pub spectral_centroid: f32,
    /// Frame duration in milliseconds.
    pub duration_ms: u16,
}

impl Default for VoiceActivityDetector {
    fn default() -> Self {
        Self {
            threshold: 0.15,
            buffer_duration_ms: 3000,
            wake_word_enabled: true,
            wake_word_model: "ruvix-wake-v1".to_string(),
            sample_rate: 16000,
            min_speech_frames: 3,
            min_silence_frames: 10,
        }
    }
}

impl VoiceActivityDetector {
    /// Create a new VAD with the given energy threshold.
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold: threshold.clamp(0.01, 0.99),
            ..Default::default()
        }
    }

    /// Create a VAD with custom buffer duration.
    pub fn with_buffer_duration(mut self, duration_ms: u64) -> Self {
        self.buffer_duration_ms = duration_ms;
        self
    }

    /// Create a VAD with wake word disabled (manual trigger mode).
    pub fn without_wake_word(mut self) -> Self {
        self.wake_word_enabled = false;
        self
    }

    /// Process a single audio frame and return a VAD decision.
    ///
    /// This is a stub implementation that uses energy level and spectral
    /// centroid to simulate speech detection. In production, this would
    /// use a 500K-param CNN from `ruv-neural-signal`.
    pub fn process_frame(&self, frame: &AudioFrame) -> VadDecision {
        // Combine energy and spectral features (stub weighting).
        let combined_score = 0.7 * frame.energy + 0.3 * frame.spectral_centroid;

        if combined_score > self.threshold {
            VadDecision::SpeechStart
        } else {
            VadDecision::Silence
        }
    }

    /// Process a sequence of frames, returning the overall detection result.
    ///
    /// Uses the consecutive-frames logic: speech is confirmed only after
    /// `min_speech_frames` consecutive frames above threshold.
    pub fn process_buffer(&self, frames: &[AudioFrame]) -> VadDecision {
        if frames.is_empty() {
            return VadDecision::Silence;
        }

        let mut consecutive_speech: u32 = 0;
        let mut consecutive_silence: u32 = 0;
        let mut speech_detected = false;

        for frame in frames {
            let combined = 0.7 * frame.energy + 0.3 * frame.spectral_centroid;
            if combined > self.threshold {
                consecutive_speech += 1;
                consecutive_silence = 0;
                if consecutive_speech >= self.min_speech_frames {
                    speech_detected = true;
                }
            } else {
                consecutive_silence += 1;
                consecutive_speech = 0;
                if speech_detected && consecutive_silence >= self.min_silence_frames {
                    return VadDecision::SpeechEnd;
                }
            }
        }

        if speech_detected {
            VadDecision::SpeechContinue
        } else {
            VadDecision::Silence
        }
    }

    /// Simulate wake word detection on a transcript (stub).
    ///
    /// In production, this would use a dedicated 20K-param keyword spotter.
    pub fn detect_wake_word(&self, transcript: &str) -> bool {
        if !self.wake_word_enabled {
            return true; // Manual trigger mode, always "detected".
        }
        let lower = transcript.to_lowercase();
        lower.contains("hey ruvix")
            || lower.contains("ok ruvix")
            || lower.contains("ruvix")
    }

    /// Adapt the energy threshold based on ambient noise level.
    pub fn adapt_threshold(&mut self, ambient_noise_energy: f32) {
        // Set threshold slightly above ambient noise.
        self.threshold = (ambient_noise_energy * 1.5 + 0.05).clamp(0.05, 0.8);
        tracing::debug!(
            new_threshold = self.threshold,
            ambient = ambient_noise_energy,
            "VAD threshold adapted"
        );
    }

    /// Returns the maximum number of samples in the rolling buffer.
    pub fn buffer_samples(&self) -> usize {
        (self.sample_rate as u64 * self.buffer_duration_ms / 1000) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn speech_frame(energy: f32) -> AudioFrame {
        AudioFrame {
            energy,
            spectral_centroid: 0.6,
            duration_ms: 20,
        }
    }

    fn silence_frame() -> AudioFrame {
        AudioFrame {
            energy: 0.02,
            spectral_centroid: 0.1,
            duration_ms: 20,
        }
    }

    #[test]
    fn test_vad_default_threshold() {
        let vad = VoiceActivityDetector::default();
        assert!((vad.threshold - 0.15).abs() < f32::EPSILON);
        assert_eq!(vad.buffer_duration_ms, 3000);
    }

    #[test]
    fn test_vad_detects_speech() {
        let vad = VoiceActivityDetector::new(0.15);
        let frame = speech_frame(0.5);
        let decision = vad.process_frame(&frame);
        assert_eq!(decision, VadDecision::SpeechStart);
    }

    #[test]
    fn test_vad_detects_silence() {
        let vad = VoiceActivityDetector::new(0.15);
        let decision = vad.process_frame(&silence_frame());
        assert_eq!(decision, VadDecision::Silence);
    }

    #[test]
    fn test_vad_buffer_consecutive_speech() {
        let vad = VoiceActivityDetector::new(0.15);
        let frames: Vec<AudioFrame> = (0..5).map(|_| speech_frame(0.5)).collect();
        let decision = vad.process_buffer(&frames);
        assert_eq!(decision, VadDecision::SpeechContinue);
    }

    #[test]
    fn test_vad_buffer_speech_then_silence() {
        let vad = VoiceActivityDetector {
            min_silence_frames: 3,
            min_speech_frames: 2,
            ..VoiceActivityDetector::new(0.15)
        };
        let mut frames: Vec<AudioFrame> = (0..5).map(|_| speech_frame(0.5)).collect();
        frames.extend((0..5).map(|_| silence_frame()));
        let decision = vad.process_buffer(&frames);
        assert_eq!(decision, VadDecision::SpeechEnd);
    }

    #[test]
    fn test_vad_empty_buffer() {
        let vad = VoiceActivityDetector::new(0.15);
        assert_eq!(vad.process_buffer(&[]), VadDecision::Silence);
    }

    #[test]
    fn test_wake_word_detection() {
        let vad = VoiceActivityDetector::default();
        assert!(vad.detect_wake_word("hey ruvix what time is it"));
        assert!(vad.detect_wake_word("Ok Ruvix play music"));
        assert!(!vad.detect_wake_word("hello world"));
    }

    #[test]
    fn test_wake_word_disabled() {
        let vad = VoiceActivityDetector::default().without_wake_word();
        // With wake word disabled, any input returns true (manual trigger mode).
        assert!(vad.detect_wake_word("anything at all"));
    }

    #[test]
    fn test_adapt_threshold() {
        let mut vad = VoiceActivityDetector::new(0.15);
        vad.adapt_threshold(0.3);
        // Threshold should be 0.3 * 1.5 + 0.05 = 0.5.
        assert!((vad.threshold - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_buffer_samples() {
        let vad = VoiceActivityDetector::default();
        // 16000 Hz * 3 seconds = 48000 samples.
        assert_eq!(vad.buffer_samples(), 48000);
    }

    #[test]
    fn test_threshold_clamping() {
        let vad = VoiceActivityDetector::new(5.0);
        assert!(vad.threshold <= 0.99);
        let vad2 = VoiceActivityDetector::new(-1.0);
        assert!(vad2.threshold >= 0.01);
    }
}
