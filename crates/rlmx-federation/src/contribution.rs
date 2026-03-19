//! User contributions: pseudonymous, domain-scoped pattern packages.
//!
//! A `Contribution` represents an anonymized set of patterns from a single user
//! for a single domain within a federation cycle. Contributions use pseudonymous
//! keys that are NOT linkable to the user's identity.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use rlmx_cognitive::sona::LoraDelta;
use rlmx_cognitive::voice_patterns::AnonymizedPattern;
use rlmx_kernel::LifeDomain;

/// A pseudonymous contribution from a single user for a single domain.
///
/// The `user_pseudonym` is a contribution-specific key that cannot be linked
/// back to the user's real identity. Each cycle generates a fresh pseudonym.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contribution {
    /// Unique identifier for this contribution.
    pub id: Uuid,
    /// Pseudonymous key for this contributor (not linkable to user identity).
    pub user_pseudonym: String,
    /// The life domain this contribution covers.
    pub domain: LifeDomain,
    /// Anonymized patterns included in this contribution.
    pub patterns: Vec<AnonymizedPattern>,
    /// Optional LoRA delta computed from the user's local adaptation.
    pub lora_delta: Option<LoraDelta>,
    /// When the anonymization was performed.
    pub anonymized_at: DateTime<Utc>,
    /// Aggregate quality score across all contributed patterns.
    pub aggregate_quality: f32,
    /// Coarse geographic bucket (e.g., "US-West", "EU-Central").
    pub region_bucket: Option<String>,
    /// Ed25519 signature hex over the serialized contribution body.
    pub signature: Option<String>,
}

impl Contribution {
    /// Create a new contribution from anonymized patterns.
    pub fn new(
        user_pseudonym: String,
        domain: LifeDomain,
        patterns: Vec<AnonymizedPattern>,
        lora_delta: Option<LoraDelta>,
    ) -> Self {
        let aggregate_quality = if patterns.is_empty() {
            0.0
        } else {
            let sum: f32 = patterns.iter().map(|p| p.result_quality).sum();
            sum / patterns.len() as f32
        };

        Self {
            id: Uuid::new_v4(),
            user_pseudonym,
            domain,
            patterns,
            lora_delta,
            anonymized_at: Utc::now(),
            aggregate_quality,
            region_bucket: None,
            signature: None,
        }
    }

    /// Set the coarse geographic region bucket.
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region_bucket = Some(region.into());
        self
    }

    /// Set the cryptographic signature.
    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = Some(signature.into());
        self
    }

    /// Number of patterns in this contribution.
    pub fn pattern_count(&self) -> usize {
        self.patterns.len()
    }

    /// Validate that this contribution meets basic integrity requirements.
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.user_pseudonym.is_empty() {
            return Err(crate::error::FederationError::ContributionRejected {
                reason: "empty pseudonym".into(),
            });
        }
        if self.patterns.is_empty() {
            return Err(crate::error::FederationError::ContributionRejected {
                reason: "no patterns provided".into(),
            });
        }
        Ok(())
    }

    /// Validate the contribution's Ed25519 signature against a public key.
    ///
    /// Returns `true` if:
    /// - The contribution has a signature AND it verifies against `pub_key`
    /// - The contribution has no signature (unsigned contributions are allowed)
    ///
    /// Returns `false` if the signature is present but does not verify.
    pub fn validate_with_key(&self, pub_key: &[u8]) -> bool {
        use ed25519_dalek::{Signature, VerifyingKey};

        let Some(ref sig_hex) = self.signature else {
            // No signature present -- unsigned contribution, accepted
            return true;
        };

        // Decode hex signature
        let Ok(sig_bytes) = hex::decode(sig_hex) else {
            return false;
        };
        let Ok(signature) = Signature::try_from(sig_bytes.as_slice()) else {
            return false;
        };

        // Decode public key (32 bytes)
        let Ok(key_bytes): Result<[u8; 32], _> = pub_key.try_into() else {
            return false;
        };
        let Ok(verifying_key) = VerifyingKey::from_bytes(&key_bytes) else {
            return false;
        };

        // Build the message: serialize contribution body excluding signature
        let body = serde_json::json!({
            "id": self.id,
            "user_pseudonym": self.user_pseudonym,
            "domain": self.domain,
            "patterns": self.patterns,
            "lora_delta": self.lora_delta,
            "anonymized_at": self.anonymized_at,
            "aggregate_quality": self.aggregate_quality,
            "region_bucket": self.region_bucket,
        });
        let Ok(body_bytes) = serde_json::to_vec(&body) else {
            return false;
        };

        use ed25519_dalek::Verifier;
        verifying_key.verify(&body_bytes, &signature).is_ok()
    }
}

/// Generate a pseudonymous contribution key from a user seed and cycle number.
///
/// Uses SHA-256 to derive a one-way pseudonym that cannot be reversed to
/// recover the original user identity.
pub fn generate_pseudonym(user_seed: &[u8], cycle_number: u64) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(user_seed);
    hasher.update(cycle_number.to_le_bytes());
    let hash = hasher.finalize();
    hex::encode(&hash[..16]) // 128-bit pseudonym
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rlmx_cognitive::voice_patterns::Modality;

    fn make_pattern() -> AnonymizedPattern {
        AnonymizedPattern {
            id: Uuid::new_v4(),
            sanitized_embedding: vec![0.1; 64],
            actions_taken: vec!["test_action".into()],
            result_quality: 0.85,
            emotion_bucket: None,
            noisy_urgency: 0.5,
            noisy_satisfaction: Some(0.7),
            interaction_modality: Modality::Voice,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_contribution_creation() {
        let patterns = vec![make_pattern(), make_pattern()];
        let contrib = Contribution::new("pseudo123".into(), LifeDomain::Finance, patterns, None);
        assert_eq!(contrib.pattern_count(), 2);
        assert!((contrib.aggregate_quality - 0.85).abs() < 0.01);
        assert_eq!(contrib.domain, LifeDomain::Finance);
    }

    #[test]
    fn test_contribution_validation_empty_pseudonym() {
        let contrib = Contribution::new("".into(), LifeDomain::Health, vec![make_pattern()], None);
        assert!(contrib.validate().is_err());
    }

    #[test]
    fn test_contribution_validation_no_patterns() {
        let contrib = Contribution::new("pseudo".into(), LifeDomain::Health, vec![], None);
        assert!(contrib.validate().is_err());
    }

    #[test]
    fn test_contribution_with_region_and_signature() {
        let contrib = Contribution::new(
            "pseudo".into(),
            LifeDomain::Shopping,
            vec![make_pattern()],
            None,
        )
        .with_region("US-West")
        .with_signature("deadbeef");
        assert_eq!(contrib.region_bucket, Some("US-West".into()));
        assert_eq!(contrib.signature, Some("deadbeef".into()));
    }

    #[test]
    fn test_generate_pseudonym_deterministic() {
        let p1 = generate_pseudonym(b"user-seed-42", 1);
        let p2 = generate_pseudonym(b"user-seed-42", 1);
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_generate_pseudonym_varies_by_cycle() {
        let p1 = generate_pseudonym(b"user-seed-42", 1);
        let p2 = generate_pseudonym(b"user-seed-42", 2);
        assert_ne!(p1, p2);
    }

    #[test]
    fn test_generate_pseudonym_varies_by_user() {
        let p1 = generate_pseudonym(b"user-a", 1);
        let p2 = generate_pseudonym(b"user-b", 1);
        assert_ne!(p1, p2);
    }
}
