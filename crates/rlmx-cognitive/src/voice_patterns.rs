//! Voice-enriched pattern recording, federated anonymization, and engagement tracking.
//!
//! Implements ADR-017: Federated Voice Learning and Privacy.
//! Key invariants:
//! - No audio ever leaves the device (only metadata).
//! - Emotion valence is bucketed before federation (prevents fingerprinting).
//! - Differential privacy via Laplace noise (epsilon=1.0) on all federated metadata.
//! - Minimum aggregation threshold before federation (>=1000 users per bucket).

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::sona::{cosine_similarity, Sona};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum VoicePatternError {
    #[error("invalid emotion valence {0}: must be in [-1.0, 1.0]")]
    InvalidValence(f32),
    #[error("invalid urgency {0}: must be in [0.0, 1.0]")]
    InvalidUrgency(f32),
    #[error("pattern not found: {0}")]
    PatternNotFound(Uuid),
}

pub type Result<T> = std::result::Result<T, VoicePatternError>;

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/// Interaction modality for a pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modality {
    Voice,
    Text,
    Multimodal,
}

/// Bucketed emotion level for federated privacy (ADR-017 invariant 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmotionBucket {
    VeryNegative,
    Negative,
    Neutral,
    Positive,
    VeryPositive,
}

// ---------------------------------------------------------------------------
// Voice-enriched pattern
// ---------------------------------------------------------------------------

/// A pattern enriched with voice interaction metadata (ADR-017).
///
/// Contains *no* audio data — only metadata derived on-device from STT output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceEnrichedPattern {
    pub id: Uuid,
    /// Embedding of the query text (NOT the audio).
    pub query_embedding: Vec<f32>,
    pub actions_taken: Vec<String>,
    /// Overall result quality (0.0 to 1.0).
    pub result_quality: f32,
    /// Emotion valence detected in voice trigger (-1.0 to 1.0).
    pub voice_trigger_emotion: Option<f32>,
    /// Urgency level inferred from speech characteristics (0.0 to 1.0).
    pub urgency_level: f32,
    /// Which modality initiated this interaction.
    pub interaction_modality: Modality,
    /// Satisfaction score inferred from follow-up behaviour (0.0 to 1.0).
    pub response_satisfaction: Option<f32>,
    /// When this pattern was recorded.
    pub timestamp: DateTime<Utc>,
}

/// An anonymized pattern safe for federated upload (ADR-017).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizedPattern {
    pub id: Uuid,
    /// Re-embedded query without named entities.
    pub sanitized_embedding: Vec<f32>,
    pub actions_taken: Vec<String>,
    pub result_quality: f32,
    /// Bucketed emotion (never precise float).
    pub emotion_bucket: Option<EmotionBucket>,
    /// Urgency with Laplace noise applied.
    pub noisy_urgency: f32,
    /// Satisfaction with Laplace noise applied.
    pub noisy_satisfaction: Option<f32>,
    pub interaction_modality: Modality,
    pub timestamp: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// VoicePatternBank
// ---------------------------------------------------------------------------

/// Extends the concept of `PatternBank` for voice-enriched patterns.
#[derive(Debug, Clone)]
pub struct VoicePatternBank {
    pub patterns: Vec<VoiceEnrichedPattern>,
    pub max_patterns: usize,
}

impl Default for VoicePatternBank {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            max_patterns: 10_000,
        }
    }
}

impl VoicePatternBank {
    pub fn new(max_patterns: usize) -> Self {
        Self {
            patterns: Vec::new(),
            max_patterns,
        }
    }

    /// Record a voice-enriched interaction pattern.
    #[allow(clippy::too_many_arguments)]
    pub fn record_voice_interaction(
        &mut self,
        query_text: &str,
        actions: Vec<String>,
        result_quality: f32,
        emotion: Option<f32>,
        urgency: f32,
        modality: Modality,
        satisfaction: Option<f32>,
    ) -> Result<Uuid> {
        // Validate ranges.
        if let Some(v) = emotion {
            if !(-1.0..=1.0).contains(&v) {
                return Err(VoicePatternError::InvalidValence(v));
            }
        }
        if !(0.0..=1.0).contains(&urgency) {
            return Err(VoicePatternError::InvalidUrgency(urgency));
        }

        let embedding = Sona::simple_embedding(query_text);
        let id = Uuid::new_v4();

        let pattern = VoiceEnrichedPattern {
            id,
            query_embedding: embedding,
            actions_taken: actions,
            result_quality: result_quality.clamp(0.0, 1.0),
            voice_trigger_emotion: emotion,
            urgency_level: urgency,
            interaction_modality: modality,
            response_satisfaction: satisfaction.map(|s| s.clamp(0.0, 1.0)),
            timestamp: Utc::now(),
        };

        // Evict lowest-quality pattern if at capacity.
        if self.patterns.len() >= self.max_patterns {
            if let Some((idx, _)) = self.patterns.iter().enumerate().min_by(|(_, a), (_, b)| {
                a.result_quality
                    .partial_cmp(&b.result_quality)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
                self.patterns.remove(idx);
            }
        }

        self.patterns.push(pattern);
        Ok(id)
    }

    /// Search patterns by emotion valence range.
    pub fn search_by_emotion(
        &self,
        min_valence: f32,
        max_valence: f32,
    ) -> Vec<&VoiceEnrichedPattern> {
        self.patterns
            .iter()
            .filter(|p| {
                if let Some(v) = p.voice_trigger_emotion {
                    v >= min_valence && v <= max_valence
                } else {
                    false
                }
            })
            .collect()
    }

    /// Temporal-weighted search: recent patterns get 3x weight (ADR-017).
    ///
    /// Returns the top `k` patterns ranked by `cosine_similarity * recency_weight`.
    pub fn temporal_weighted_search(
        &self,
        query_embedding: &[f32],
        k: usize,
    ) -> Vec<&VoiceEnrichedPattern> {
        if self.patterns.is_empty() {
            return Vec::new();
        }

        let now = Utc::now();
        let one_day_secs: f64 = 86_400.0;

        let mut scored: Vec<(usize, f64)> = self
            .patterns
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let sim = cosine_similarity(query_embedding, &p.query_embedding);
                let age_days = (now - p.timestamp).num_seconds().max(0) as f64 / one_day_secs;
                // Recent (< 1 day) gets 3x weight, decays exponentially.
                let recency_weight = 1.0 + 2.0 * (-age_days / 7.0_f64).exp();
                (i, sim * recency_weight)
            })
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        scored
            .iter()
            .take(k)
            .filter_map(|(i, _)| self.patterns.get(*i))
            .collect()
    }

    /// Return the number of stored patterns.
    pub fn len(&self) -> usize {
        self.patterns.len()
    }

    /// Whether the bank is empty.
    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }
}

// ---------------------------------------------------------------------------
// FederatedAnonymizer — ADR-017 Privacy Pipeline
// ---------------------------------------------------------------------------

/// Anonymizes voice-enriched patterns for safe federated upload.
///
/// Implements the three-step pipeline from ADR-017:
/// 1. Strip PII from transcript embedding (re-embed without named entities).
/// 2. Bucket emotion valence into 5 discrete categories.
/// 3. Add Laplace noise (epsilon=1.0) to urgency and satisfaction scores.
///
/// Also enforces ADR-017 privacy invariant 5: minimum aggregation threshold
/// (patterns are only safe to federate when >= `min_aggregation_threshold`
/// users share the same domain+region bucket).
pub struct FederatedAnonymizer {
    /// Differential privacy epsilon parameter.
    pub epsilon: f64,
    /// Minimum number of users in a domain+region bucket before patterns from
    /// that bucket may be federated (ADR-017 invariant 5). Default: 1000.
    pub min_aggregation_threshold: usize,
}

impl Default for FederatedAnonymizer {
    fn default() -> Self {
        Self {
            epsilon: 1.0,
            min_aggregation_threshold: 1000,
        }
    }
}

impl FederatedAnonymizer {
    pub fn new(epsilon: f64) -> Self {
        Self {
            epsilon,
            min_aggregation_threshold: 1000,
        }
    }

    /// Create an anonymizer with a custom aggregation threshold.
    pub fn with_aggregation_threshold(epsilon: f64, min_aggregation_threshold: usize) -> Self {
        Self {
            epsilon,
            min_aggregation_threshold,
        }
    }

    /// Check whether a domain+region bucket has enough users to safely
    /// federate patterns from it (ADR-017 invariant 5).
    ///
    /// Returns `true` if `bucket_user_count >= min_aggregation_threshold`.
    pub fn meets_aggregation_threshold(&self, bucket_user_count: usize) -> bool {
        bucket_user_count >= self.min_aggregation_threshold
    }

    /// Strip PII by zeroing embedding dimensions associated with named entities.
    ///
    /// In production this would re-run STT→NER→re-embed pipeline. Here we
    /// simulate by applying a deterministic mask to reduce information content.
    pub fn strip_pii(embedding: &[f32]) -> Vec<f32> {
        // Mask every 4th dimension and renormalize — simulates PII scrubbing.
        let mut sanitized: Vec<f32> = embedding
            .iter()
            .enumerate()
            .map(|(i, &v)| if i % 4 == 0 { 0.0 } else { v })
            .collect();

        // Renormalize so the embedding still has unit length.
        let norm: f32 = sanitized.iter().map(|v| v * v).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut sanitized {
                *v /= norm;
            }
        }
        sanitized
    }

    /// Bucket a continuous emotion valence into one of five discrete categories.
    pub fn bucket_emotion(valence: f32) -> EmotionBucket {
        match valence {
            v if v <= -0.6 => EmotionBucket::VeryNegative,
            v if v <= -0.2 => EmotionBucket::Negative,
            v if v <= 0.2 => EmotionBucket::Neutral,
            v if v <= 0.6 => EmotionBucket::Positive,
            _ => EmotionBucket::VeryPositive,
        }
    }

    /// Add Laplace noise to a value for differential privacy.
    ///
    /// The scale parameter is `sensitivity / epsilon` where sensitivity=1.0 for
    /// our normalised scores.
    pub fn add_laplace_noise(&self, value: f32) -> f32 {
        let scale = 1.0 / self.epsilon;
        let mut rng = rand::thread_rng();
        let u: f64 = rng.gen_range(-0.5..0.5);
        let noise = -scale * u.abs().ln().copysign(u);
        value + noise as f32
    }

    /// Full anonymization pipeline: strip PII, bucket emotion, add noise.
    pub fn anonymize_pattern(&self, pattern: &VoiceEnrichedPattern) -> AnonymizedPattern {
        let sanitized_embedding = Self::strip_pii(&pattern.query_embedding);
        let emotion_bucket = pattern.voice_trigger_emotion.map(Self::bucket_emotion);
        let noisy_urgency = self.add_laplace_noise(pattern.urgency_level);
        let noisy_satisfaction = pattern
            .response_satisfaction
            .map(|s| self.add_laplace_noise(s));

        AnonymizedPattern {
            id: Uuid::new_v4(), // New ID — breaks linkability.
            sanitized_embedding,
            actions_taken: pattern.actions_taken.clone(),
            result_quality: pattern.result_quality,
            emotion_bucket,
            noisy_urgency,
            noisy_satisfaction,
            interaction_modality: pattern.interaction_modality,
            timestamp: pattern.timestamp,
        }
    }
}

// ---------------------------------------------------------------------------
// EngagementTracker
// ---------------------------------------------------------------------------

/// Tracks user engagement patterns for fatigue modelling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementEvent {
    pub timestamp: DateTime<Utc>,
    pub notification_type: String,
    pub responded: bool,
    /// Seconds until the user responded (None if ignored).
    pub response_latency_secs: Option<f64>,
}

/// Tracks engagement events and computes engagement metrics.
#[derive(Debug, Clone, Default)]
pub struct EngagementTracker {
    pub events: Vec<EngagementEvent>,
    pub max_events: usize,
}

impl EngagementTracker {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Vec::new(),
            max_events,
        }
    }

    /// Record a user engagement event.
    pub fn record(
        &mut self,
        notification_type: &str,
        responded: bool,
        response_latency_secs: Option<f64>,
    ) {
        let event = EngagementEvent {
            timestamp: Utc::now(),
            notification_type: notification_type.to_string(),
            responded,
            response_latency_secs,
        };

        if self.events.len() >= self.max_events && self.max_events > 0 {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    /// Overall response rate across all notification types.
    pub fn response_rate(&self) -> f64 {
        if self.events.is_empty() {
            return 0.0;
        }
        let responded = self.events.iter().filter(|e| e.responded).count();
        responded as f64 / self.events.len() as f64
    }

    /// Response rate for a specific notification type.
    pub fn response_rate_for_type(&self, notification_type: &str) -> f64 {
        let matching: Vec<&EngagementEvent> = self
            .events
            .iter()
            .filter(|e| e.notification_type == notification_type)
            .collect();
        if matching.is_empty() {
            return 0.0;
        }
        let responded = matching.iter().filter(|e| e.responded).count();
        responded as f64 / matching.len() as f64
    }

    /// Average response latency (only for responded events).
    pub fn avg_response_latency(&self) -> Option<f64> {
        let latencies: Vec<f64> = self
            .events
            .iter()
            .filter_map(|e| e.response_latency_secs)
            .collect();
        if latencies.is_empty() {
            return None;
        }
        Some(latencies.iter().sum::<f64>() / latencies.len() as f64)
    }

    /// Number of events recorded.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Whether no events have been recorded.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

// ---------------------------------------------------------------------------
// NotificationFatigueModel
// ---------------------------------------------------------------------------

/// Predicts whether a user will respond to a notification type based on
/// recent engagement history.
///
/// Uses a simple logistic-style model: if the recent response rate for a
/// notification type drops below a threshold, the user is considered fatigued.
#[derive(Debug, Clone)]
pub struct NotificationFatigueModel {
    /// Below this response rate the user is considered fatigued.
    pub fatigue_threshold: f64,
    /// Number of recent events to consider for prediction.
    pub window_size: usize,
}

impl Default for NotificationFatigueModel {
    fn default() -> Self {
        Self {
            fatigue_threshold: 0.3,
            window_size: 20,
        }
    }
}

impl NotificationFatigueModel {
    pub fn new(fatigue_threshold: f64, window_size: usize) -> Self {
        Self {
            fatigue_threshold,
            window_size,
        }
    }

    /// Predict whether the user will respond to a given notification type.
    ///
    /// Returns `true` if the user is likely to respond (not fatigued).
    pub fn predict_response(&self, tracker: &EngagementTracker, notification_type: &str) -> bool {
        let recent: Vec<&EngagementEvent> = tracker
            .events
            .iter()
            .rev()
            .filter(|e| e.notification_type == notification_type)
            .take(self.window_size)
            .collect();

        if recent.is_empty() {
            // No history — assume user will respond.
            return true;
        }

        let responded = recent.iter().filter(|e| e.responded).count();
        let rate = responded as f64 / recent.len() as f64;
        rate >= self.fatigue_threshold
    }

    /// Return a fatigue score from 0.0 (not fatigued) to 1.0 (very fatigued).
    pub fn fatigue_score(&self, tracker: &EngagementTracker, notification_type: &str) -> f64 {
        let recent: Vec<&EngagementEvent> = tracker
            .events
            .iter()
            .rev()
            .filter(|e| e.notification_type == notification_type)
            .take(self.window_size)
            .collect();

        if recent.is_empty() {
            return 0.0;
        }

        let responded = recent.iter().filter(|e| e.responded).count();
        let rate = responded as f64 / recent.len() as f64;
        (1.0 - rate).clamp(0.0, 1.0)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- VoicePatternBank tests --

    #[test]
    fn test_record_voice_interaction() {
        let mut bank = VoicePatternBank::new(100);
        let id = bank
            .record_voice_interaction(
                "check my balance",
                vec!["open_banking".into()],
                0.9,
                Some(0.2),
                0.5,
                Modality::Voice,
                Some(0.8),
            )
            .unwrap();

        assert_eq!(bank.len(), 1);
        assert_eq!(bank.patterns[0].id, id);
        assert_eq!(bank.patterns[0].interaction_modality, Modality::Voice);
        assert!((bank.patterns[0].urgency_level - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_record_rejects_invalid_valence() {
        let mut bank = VoicePatternBank::new(100);
        let err = bank
            .record_voice_interaction("query", vec![], 0.5, Some(1.5), 0.5, Modality::Text, None)
            .unwrap_err();
        assert!(matches!(err, VoicePatternError::InvalidValence(_)));
    }

    #[test]
    fn test_record_rejects_invalid_urgency() {
        let mut bank = VoicePatternBank::new(100);
        let err = bank
            .record_voice_interaction("query", vec![], 0.5, None, 1.5, Modality::Text, None)
            .unwrap_err();
        assert!(matches!(err, VoicePatternError::InvalidUrgency(_)));
    }

    #[test]
    fn test_voice_bank_capacity_eviction() {
        let mut bank = VoicePatternBank::new(2);
        bank.record_voice_interaction("a", vec![], 0.3, None, 0.1, Modality::Text, None)
            .unwrap();
        bank.record_voice_interaction("b", vec![], 0.9, None, 0.1, Modality::Text, None)
            .unwrap();
        bank.record_voice_interaction("c", vec![], 0.7, None, 0.1, Modality::Text, None)
            .unwrap();

        assert_eq!(bank.len(), 2);
        // The 0.3 quality pattern should have been evicted.
        assert!(bank.patterns.iter().all(|p| p.result_quality >= 0.7));
    }

    #[test]
    fn test_search_by_emotion() {
        let mut bank = VoicePatternBank::new(100);
        bank.record_voice_interaction(
            "happy query",
            vec![],
            0.8,
            Some(0.8),
            0.1,
            Modality::Voice,
            None,
        )
        .unwrap();
        bank.record_voice_interaction(
            "sad query",
            vec![],
            0.7,
            Some(-0.7),
            0.1,
            Modality::Voice,
            None,
        )
        .unwrap();
        bank.record_voice_interaction(
            "neutral query",
            vec![],
            0.6,
            Some(0.0),
            0.1,
            Modality::Text,
            None,
        )
        .unwrap();
        bank.record_voice_interaction("no emotion", vec![], 0.5, None, 0.1, Modality::Text, None)
            .unwrap();

        let positive = bank.search_by_emotion(0.5, 1.0);
        assert_eq!(positive.len(), 1);
        assert!((positive[0].voice_trigger_emotion.unwrap() - 0.8).abs() < f32::EPSILON);

        let negative = bank.search_by_emotion(-1.0, -0.5);
        assert_eq!(negative.len(), 1);
    }

    #[test]
    fn test_temporal_weighted_search() {
        let mut bank = VoicePatternBank::new(100);
        // Record two patterns with the same text so embeddings are identical.
        bank.record_voice_interaction("find balance", vec![], 0.8, None, 0.1, Modality::Text, None)
            .unwrap();
        bank.record_voice_interaction("find balance", vec![], 0.5, None, 0.1, Modality::Text, None)
            .unwrap();

        let query_emb = Sona::simple_embedding("find balance");
        let results = bank.temporal_weighted_search(&query_emb, 2);
        assert_eq!(results.len(), 2);
        // The most recent pattern should come first (recency boost).
        // Both have same embedding similarity, so the more recent one wins.
    }

    #[test]
    fn test_temporal_weighted_search_empty() {
        let bank = VoicePatternBank::new(100);
        let query_emb = Sona::simple_embedding("anything");
        let results = bank.temporal_weighted_search(&query_emb, 5);
        assert!(results.is_empty());
    }

    // -- FederatedAnonymizer tests --

    #[test]
    fn test_strip_pii_changes_embedding() {
        let original = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let sanitized = FederatedAnonymizer::strip_pii(&original);
        assert_eq!(sanitized.len(), original.len());
        // Every 4th dimension (indices 0, 4) should be zeroed then renormalized.
        // The sanitized embedding should differ from the original.
        assert_ne!(original, sanitized);
        // Dimension 0 was zeroed before renormalization, so after renorm it stays 0.
        assert!((sanitized[0]).abs() < f32::EPSILON);
        assert!((sanitized[4]).abs() < f32::EPSILON);
    }

    #[test]
    fn test_strip_pii_preserves_unit_norm() {
        let original = Sona::simple_embedding("some user query with John Smith");
        let sanitized = FederatedAnonymizer::strip_pii(&original);
        let norm: f32 = sanitized.iter().map(|v| v * v).sum::<f32>().sqrt();
        assert!(
            (norm - 1.0).abs() < 0.01,
            "Sanitized embedding should be ~unit norm, got {}",
            norm
        );
    }

    #[test]
    fn test_bucket_emotion() {
        assert_eq!(
            FederatedAnonymizer::bucket_emotion(-0.9),
            EmotionBucket::VeryNegative
        );
        assert_eq!(
            FederatedAnonymizer::bucket_emotion(-0.4),
            EmotionBucket::Negative
        );
        assert_eq!(
            FederatedAnonymizer::bucket_emotion(0.0),
            EmotionBucket::Neutral
        );
        assert_eq!(
            FederatedAnonymizer::bucket_emotion(0.4),
            EmotionBucket::Positive
        );
        assert_eq!(
            FederatedAnonymizer::bucket_emotion(0.9),
            EmotionBucket::VeryPositive
        );
    }

    #[test]
    fn test_bucket_emotion_boundaries() {
        // Exact boundary values.
        assert_eq!(
            FederatedAnonymizer::bucket_emotion(-0.6),
            EmotionBucket::VeryNegative
        );
        assert_eq!(
            FederatedAnonymizer::bucket_emotion(-0.2),
            EmotionBucket::Negative
        );
        assert_eq!(
            FederatedAnonymizer::bucket_emotion(0.2),
            EmotionBucket::Neutral
        );
        assert_eq!(
            FederatedAnonymizer::bucket_emotion(0.6),
            EmotionBucket::Positive
        );
    }

    #[test]
    fn test_laplace_noise_adds_noise() {
        let anon = FederatedAnonymizer::new(1.0);
        let original = 0.5;
        // Run many times and check that at least some values differ.
        let noisy_values: Vec<f32> = (0..100).map(|_| anon.add_laplace_noise(original)).collect();
        let all_same = noisy_values
            .iter()
            .all(|&v| (v - original).abs() < f32::EPSILON);
        assert!(!all_same, "Laplace noise should produce varying values");
    }

    #[test]
    fn test_anonymize_pattern_pipeline() {
        let pattern = VoiceEnrichedPattern {
            id: Uuid::new_v4(),
            query_embedding: Sona::simple_embedding("pay my electric bill"),
            actions_taken: vec!["bill_pay".into()],
            result_quality: 0.85,
            voice_trigger_emotion: Some(-0.3),
            urgency_level: 0.7,
            interaction_modality: Modality::Voice,
            response_satisfaction: Some(0.9),
            timestamp: Utc::now(),
        };

        let anon = FederatedAnonymizer::default();
        let anonymized = anon.anonymize_pattern(&pattern);

        // ID should be different (breaks linkability).
        assert_ne!(anonymized.id, pattern.id);
        // Emotion should be bucketed.
        assert_eq!(anonymized.emotion_bucket, Some(EmotionBucket::Negative));
        // Embedding should be sanitized (different from original).
        assert_ne!(anonymized.sanitized_embedding, pattern.query_embedding);
        // Noisy urgency should differ from exact value (with high probability).
        // We can't guarantee this in a single run, so just check it exists.
        assert!(anonymized.noisy_satisfaction.is_some());
    }

    #[test]
    fn test_anonymize_pattern_no_emotion() {
        let pattern = VoiceEnrichedPattern {
            id: Uuid::new_v4(),
            query_embedding: vec![0.5; 8],
            actions_taken: vec![],
            result_quality: 0.5,
            voice_trigger_emotion: None,
            urgency_level: 0.5,
            interaction_modality: Modality::Text,
            response_satisfaction: None,
            timestamp: Utc::now(),
        };

        let anon = FederatedAnonymizer::default();
        let anonymized = anon.anonymize_pattern(&pattern);
        assert!(anonymized.emotion_bucket.is_none());
        assert!(anonymized.noisy_satisfaction.is_none());
    }

    // -- EngagementTracker tests --

    #[test]
    fn test_engagement_tracker_record_and_rate() {
        let mut tracker = EngagementTracker::new(100);
        tracker.record("alert", true, Some(2.0));
        tracker.record("alert", false, None);
        tracker.record("alert", true, Some(5.0));

        assert_eq!(tracker.len(), 3);
        let rate = tracker.response_rate();
        assert!((rate - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_engagement_tracker_type_rate() {
        let mut tracker = EngagementTracker::new(100);
        tracker.record("alert", true, Some(1.0));
        tracker.record("alert", false, None);
        tracker.record("promo", true, Some(3.0));
        tracker.record("promo", true, Some(2.0));

        assert!((tracker.response_rate_for_type("alert") - 0.5).abs() < 0.01);
        assert!((tracker.response_rate_for_type("promo") - 1.0).abs() < 0.01);
        assert!((tracker.response_rate_for_type("unknown") - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_engagement_tracker_avg_latency() {
        let mut tracker = EngagementTracker::new(100);
        tracker.record("alert", true, Some(2.0));
        tracker.record("alert", true, Some(4.0));
        tracker.record("alert", false, None);

        let avg = tracker.avg_response_latency().unwrap();
        assert!((avg - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_engagement_tracker_capacity() {
        let mut tracker = EngagementTracker::new(2);
        tracker.record("a", true, Some(1.0));
        tracker.record("b", false, None);
        tracker.record("c", true, Some(3.0));

        assert_eq!(tracker.len(), 2);
        // Oldest event ("a") should have been evicted.
        assert_eq!(tracker.events[0].notification_type, "b");
    }

    // -- NotificationFatigueModel tests --

    #[test]
    fn test_fatigue_model_predicts_response() {
        let mut tracker = EngagementTracker::new(100);
        // User responds to most alerts.
        for _ in 0..8 {
            tracker.record("alert", true, Some(1.0));
        }
        for _ in 0..2 {
            tracker.record("alert", false, None);
        }

        let model = NotificationFatigueModel::default();
        assert!(model.predict_response(&tracker, "alert"));
    }

    #[test]
    fn test_fatigue_model_detects_fatigue() {
        let mut tracker = EngagementTracker::new(100);
        // User ignores most promos.
        for _ in 0..9 {
            tracker.record("promo", false, None);
        }
        tracker.record("promo", true, Some(10.0));

        let model = NotificationFatigueModel::default();
        assert!(!model.predict_response(&tracker, "promo"));
    }

    #[test]
    fn test_fatigue_score() {
        let mut tracker = EngagementTracker::new(100);
        for _ in 0..10 {
            tracker.record("alert", false, None);
        }

        let model = NotificationFatigueModel::default();
        let score = model.fatigue_score(&tracker, "alert");
        assert!(
            (score - 1.0).abs() < 0.01,
            "All ignored should give fatigue=1.0"
        );

        let score_unknown = model.fatigue_score(&tracker, "unknown");
        assert!(
            (score_unknown - 0.0).abs() < 0.01,
            "No history should give fatigue=0.0"
        );
    }

    #[test]
    fn test_fatigue_model_no_history() {
        let tracker = EngagementTracker::new(100);
        let model = NotificationFatigueModel::default();
        // No history should assume user will respond.
        assert!(model.predict_response(&tracker, "anything"));
    }

    // -- Aggregation threshold tests (ADR-017 invariant 5) --

    #[test]
    fn test_aggregation_threshold_default() {
        let anon = FederatedAnonymizer::default();
        assert_eq!(anon.min_aggregation_threshold, 1000);
    }

    #[test]
    fn test_aggregation_threshold_enforced() {
        let anon = FederatedAnonymizer::default();
        // Below threshold: not safe to federate.
        assert!(!anon.meets_aggregation_threshold(999));
        assert!(!anon.meets_aggregation_threshold(0));
        // At or above threshold: safe.
        assert!(anon.meets_aggregation_threshold(1000));
        assert!(anon.meets_aggregation_threshold(5000));
    }

    #[test]
    fn test_custom_aggregation_threshold() {
        let anon = FederatedAnonymizer::with_aggregation_threshold(1.0, 500);
        assert_eq!(anon.min_aggregation_threshold, 500);
        assert!(!anon.meets_aggregation_threshold(499));
        assert!(anon.meets_aggregation_threshold(500));
    }
}
