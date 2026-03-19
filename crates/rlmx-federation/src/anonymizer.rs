//! On-device anonymization for federated learning.
//!
//! Wraps `rlmx-cognitive`'s `FederatedAnonymizer` and adds federation-specific
//! validation: aggregation threshold enforcement, PII detection, and
//! contribution-level anonymization.
//!
//! Privacy invariants (ADR-023, non-negotiable):
//! - No raw user data ever leaves the device.
//! - Patterns anonymized BEFORE leaving device.
//! - Emotion valence bucketed into 5 levels.
//! - Laplace noise epsilon=1.0 on urgency/satisfaction.
//! - No speaker embeddings federated.
//! - Minimum 1000-user aggregation threshold.

use rlmx_cognitive::sona::LoraDelta;
use rlmx_cognitive::voice_patterns::{
    AnonymizedPattern, FederatedAnonymizer, VoiceEnrichedPattern,
};
use rlmx_kernel::LifeDomain;
use tracing::{debug, warn};

use crate::contribution::Contribution;

/// Configuration for the anonymization pipeline.
#[derive(Debug, Clone)]
pub struct AnonymizationConfig {
    /// Differential privacy epsilon parameter.
    pub epsilon: f64,
    /// Minimum number of contributors before patterns can be aggregated.
    pub aggregation_threshold: usize,
    /// Maximum patterns per contribution to limit information leakage.
    pub max_patterns_per_contribution: usize,
}

impl Default for AnonymizationConfig {
    fn default() -> Self {
        Self {
            epsilon: 1.0,
            aggregation_threshold: 1000,
            max_patterns_per_contribution: 500,
        }
    }
}

/// Federation-level anonymizer that wraps the cognitive crate's `FederatedAnonymizer`
/// and adds contribution packaging.
pub struct FederationAnonymizer {
    inner: FederatedAnonymizer,
    config: AnonymizationConfig,
}

impl std::fmt::Debug for FederationAnonymizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FederationAnonymizer")
            .field("epsilon", &self.config.epsilon)
            .field("config", &self.config)
            .finish()
    }
}

impl FederationAnonymizer {
    /// Create a new federation anonymizer with the given config.
    pub fn new(config: AnonymizationConfig) -> Self {
        let inner = FederatedAnonymizer::new(config.epsilon);
        Self { inner, config }
    }

    /// Return the aggregation threshold.
    pub fn aggregation_threshold(&self) -> usize {
        self.config.aggregation_threshold
    }

    /// Anonymize a batch of voice-enriched patterns into a contribution.
    ///
    /// This is the main entry point for on-device anonymization before federation.
    /// Applies the full pipeline: strip PII, bucket emotion, add Laplace noise.
    pub fn anonymize_patterns(
        &self,
        patterns: &[VoiceEnrichedPattern],
        user_pseudonym: String,
        domain: LifeDomain,
        lora_delta: Option<LoraDelta>,
    ) -> crate::error::Result<Contribution> {
        if patterns.is_empty() {
            return Err(crate::error::FederationError::AnonymizationFailed(
                "no patterns to anonymize".into(),
            ));
        }

        // Limit the number of patterns to prevent information leakage.
        let limit = self
            .config
            .max_patterns_per_contribution
            .min(patterns.len());
        let selected = &patterns[..limit];

        debug!(
            pattern_count = selected.len(),
            domain = ?domain,
            "anonymizing patterns for federation"
        );

        let anonymized: Vec<AnonymizedPattern> = selected
            .iter()
            .map(|p| self.inner.anonymize_pattern(p))
            .collect();

        Ok(Contribution::new(
            user_pseudonym,
            domain,
            anonymized,
            lora_delta,
        ))
    }

    /// Strip PII from a single embedding (delegates to cognitive crate).
    pub fn strip_pii(embedding: &[f32]) -> Vec<f32> {
        FederatedAnonymizer::strip_pii(embedding)
    }

    /// Bucket an emotion valence into 5 discrete levels (delegates to cognitive crate).
    pub fn bucket_emotion(valence: f32) -> rlmx_cognitive::voice_patterns::EmotionBucket {
        FederatedAnonymizer::bucket_emotion(valence)
    }

    /// Add Laplace noise for differential privacy.
    pub fn add_laplace_noise(&self, value: f32) -> f32 {
        self.inner.add_laplace_noise(value)
    }

    /// Check whether a text string likely contains PII.
    ///
    /// Simple heuristic: looks for patterns resembling email addresses,
    /// phone numbers, or social security numbers.
    pub fn likely_contains_pii(text: &str) -> bool {
        // Email pattern
        if text.contains('@') && text.contains('.') {
            return true;
        }
        // Phone-like sequences (10+ consecutive digits possibly with separators)
        let digit_count = text.chars().filter(|c| c.is_ascii_digit()).count();
        if digit_count >= 9 {
            return true;
        }
        // SSN pattern (NNN-NN-NNNN)
        if text.len() == 11
            && text.chars().enumerate().all(|(i, c)| {
                if i == 3 || i == 6 {
                    c == '-'
                } else {
                    c.is_ascii_digit()
                }
            })
        {
            return true;
        }
        false
    }

    /// Validate that an anonymized pattern does not violate privacy invariants.
    pub fn validate_anonymized(pattern: &AnonymizedPattern) -> crate::error::Result<()> {
        // Ensure actions don't contain PII.
        for action in &pattern.actions_taken {
            if Self::likely_contains_pii(action) {
                warn!(action = %action, "PII detected in action string");
                return Err(crate::error::FederationError::PrivacyViolation(
                    "PII detected in anonymized pattern actions".into(),
                ));
            }
        }
        Ok(())
    }
}

impl Default for FederationAnonymizer {
    fn default() -> Self {
        Self::new(AnonymizationConfig::default())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rlmx_cognitive::voice_patterns::Modality;
    use uuid::Uuid;

    fn make_voice_pattern(emotion: Option<f32>, urgency: f32) -> VoiceEnrichedPattern {
        VoiceEnrichedPattern {
            id: Uuid::new_v4(),
            query_embedding: vec![0.1; 64],
            actions_taken: vec!["test_action".into()],
            result_quality: 0.85,
            voice_trigger_emotion: emotion,
            urgency_level: urgency,
            interaction_modality: Modality::Voice,
            response_satisfaction: Some(0.7),
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_anonymize_patterns() {
        let anon = FederationAnonymizer::default();
        let patterns = vec![make_voice_pattern(Some(0.5), 0.3)];
        let contrib = anon
            .anonymize_patterns(&patterns, "pseudo".into(), LifeDomain::Finance, None)
            .unwrap();
        assert_eq!(contrib.pattern_count(), 1);
        assert_eq!(contrib.domain, LifeDomain::Finance);
        assert_eq!(contrib.user_pseudonym, "pseudo");
    }

    #[test]
    fn test_anonymize_empty_patterns_fails() {
        let anon = FederationAnonymizer::default();
        let result = anon.anonymize_patterns(&[], "pseudo".into(), LifeDomain::Health, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_anonymize_respects_max_limit() {
        let config = AnonymizationConfig {
            max_patterns_per_contribution: 2,
            ..Default::default()
        };
        let anon = FederationAnonymizer::new(config);
        let patterns: Vec<_> = (0..5).map(|_| make_voice_pattern(None, 0.5)).collect();
        let contrib = anon
            .anonymize_patterns(&patterns, "pseudo".into(), LifeDomain::Shopping, None)
            .unwrap();
        assert_eq!(contrib.pattern_count(), 2);
    }

    #[test]
    fn test_likely_contains_pii_email() {
        assert!(FederationAnonymizer::likely_contains_pii(
            "user@example.com"
        ));
    }

    #[test]
    fn test_likely_contains_pii_phone() {
        assert!(FederationAnonymizer::likely_contains_pii(
            "call 555-123-4567"
        ));
    }

    #[test]
    fn test_likely_contains_pii_ssn() {
        assert!(FederationAnonymizer::likely_contains_pii("123-45-6789"));
    }

    #[test]
    fn test_no_pii_in_clean_text() {
        assert!(!FederationAnonymizer::likely_contains_pii(
            "pay electric bill"
        ));
    }

    #[test]
    fn test_validate_anonymized_clean() {
        use rlmx_cognitive::voice_patterns::AnonymizedPattern;
        let pattern = AnonymizedPattern {
            id: Uuid::new_v4(),
            sanitized_embedding: vec![0.1; 8],
            actions_taken: vec!["bill_pay".into()],
            result_quality: 0.8,
            emotion_bucket: None,
            noisy_urgency: 0.5,
            noisy_satisfaction: None,
            interaction_modality: Modality::Text,
            timestamp: Utc::now(),
        };
        assert!(FederationAnonymizer::validate_anonymized(&pattern).is_ok());
    }

    #[test]
    fn test_validate_anonymized_with_pii_fails() {
        use rlmx_cognitive::voice_patterns::AnonymizedPattern;
        let pattern = AnonymizedPattern {
            id: Uuid::new_v4(),
            sanitized_embedding: vec![0.1; 8],
            actions_taken: vec!["email john@example.com".into()],
            result_quality: 0.8,
            emotion_bucket: None,
            noisy_urgency: 0.5,
            noisy_satisfaction: None,
            interaction_modality: Modality::Text,
            timestamp: Utc::now(),
        };
        assert!(FederationAnonymizer::validate_anonymized(&pattern).is_err());
    }
}
