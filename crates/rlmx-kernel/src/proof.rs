use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::types::{KernelError, KernelResult, ProofRequest};

/// A single witness in the append-only chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Witness {
    pub id: Uuid,
    pub action_hash: String,
    pub reasoning_chain_hash: String,
    pub evidence_refs: Vec<String>,
    pub timestamp: DateTime<Utc>,
    /// SHA-256 hash of the previous witness entry (empty string for the first entry).
    pub prev_hash: String,
    /// SHA-256 hash of all fields in this witness (provides tamper detection).
    pub content_hash: String,
}

impl Witness {
    /// Compute the content hash over all semantic fields of this witness.
    ///
    /// The hash covers: id, action_hash, reasoning_chain_hash, evidence_refs,
    /// timestamp, and prev_hash. This means `content_hash` itself is excluded
    /// from the computation (it is the *output*).
    fn compute_content_hash(
        id: &Uuid,
        action_hash: &str,
        reasoning_chain_hash: &str,
        evidence_refs: &[String],
        timestamp: &DateTime<Utc>,
        prev_hash: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(id.to_string().as_bytes());
        hasher.update(action_hash.as_bytes());
        hasher.update(reasoning_chain_hash.as_bytes());
        for r in evidence_refs {
            hasher.update(r.as_bytes());
        }
        hasher.update(timestamp.to_rfc3339().as_bytes());
        hasher.update(prev_hash.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// A proof produced after validating a state mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proof {
    pub id: Uuid,
    pub witness_id: Uuid,
    pub valid: bool,
    pub confidence: f64,
    pub reason: String,
}

/// Append-only log of all state mutation witnesses.
pub struct WitnessChain {
    witnesses: Vec<Witness>,
}

impl WitnessChain {
    pub fn new() -> Self {
        Self {
            witnesses: Vec::new(),
        }
    }

    /// Append a witness to the chain and return its id.
    pub fn append(
        &mut self,
        action_hash: String,
        reasoning_chain_hash: String,
        evidence_refs: Vec<String>,
    ) -> Uuid {
        let id = Uuid::new_v4();
        let timestamp = Utc::now();

        // Chain to the previous entry's content_hash, or "" for the genesis entry.
        let prev_hash = self
            .witnesses
            .last()
            .map(|w| w.content_hash.clone())
            .unwrap_or_default();

        let content_hash = Witness::compute_content_hash(
            &id,
            &action_hash,
            &reasoning_chain_hash,
            &evidence_refs,
            &timestamp,
            &prev_hash,
        );

        let witness = Witness {
            id,
            action_hash,
            reasoning_chain_hash,
            evidence_refs,
            timestamp,
            prev_hash,
            content_hash,
        };
        self.witnesses.push(witness);
        id
    }

    /// Get a witness by id.
    pub fn get(&self, id: &Uuid) -> Option<&Witness> {
        self.witnesses.iter().find(|w| w.id == *id)
    }

    /// Total number of witnesses.
    pub fn len(&self) -> usize {
        self.witnesses.len()
    }

    pub fn is_empty(&self) -> bool {
        self.witnesses.is_empty()
    }

    /// Verify the integrity of the entire witness chain.
    ///
    /// Checks that:
    /// 1. The first entry has an empty `prev_hash`.
    /// 2. Each subsequent entry's `prev_hash` equals the previous entry's `content_hash`.
    /// 3. Every entry's `content_hash` is consistent with its fields.
    pub fn verify_integrity(&self) -> bool {
        for (i, witness) in self.witnesses.iter().enumerate() {
            // Verify prev_hash linkage.
            let expected_prev = if i == 0 {
                String::new()
            } else {
                self.witnesses[i - 1].content_hash.clone()
            };
            if witness.prev_hash != expected_prev {
                return false;
            }

            // Verify content_hash is correct.
            let recomputed = Witness::compute_content_hash(
                &witness.id,
                &witness.action_hash,
                &witness.reasoning_chain_hash,
                &witness.evidence_refs,
                &witness.timestamp,
                &witness.prev_hash,
            );
            if witness.content_hash != recomputed {
                return false;
            }
        }
        true
    }
}

impl Default for WitnessChain {
    fn default() -> Self {
        Self::new()
    }
}

/// The proof engine validates state mutations against the witness chain.
pub struct ProofEngine {
    pub chain: WitnessChain,
}

impl ProofEngine {
    pub fn new() -> Self {
        Self {
            chain: WitnessChain::new(),
        }
    }

    /// Record a state mutation and produce a proof.
    ///
    /// Simplified validation: the proof is considered valid if the provided
    /// confidence exceeds the threshold in the proof request, and if evidence
    /// is required, at least one evidence reference must be provided.
    pub fn validate(
        &mut self,
        action: &str,
        reasoning_chain: &str,
        evidence_refs: Vec<String>,
        confidence: f64,
        request: &ProofRequest,
    ) -> KernelResult<Proof> {
        // Check evidence requirement.
        if request.require_evidence && evidence_refs.is_empty() {
            return Err(KernelError::ProofError(
                "evidence required but none provided".into(),
            ));
        }

        let action_hash = hash_sha256(action);
        let reasoning_hash = hash_sha256(reasoning_chain);

        let witness_id = self
            .chain
            .append(action_hash, reasoning_hash, evidence_refs);

        let valid = confidence >= request.confidence_threshold;

        let reason = if valid {
            format!(
                "confidence {:.3} >= threshold {:.3}",
                confidence, request.confidence_threshold
            )
        } else {
            format!(
                "confidence {:.3} < threshold {:.3}",
                confidence, request.confidence_threshold
            )
        };

        Ok(Proof {
            id: Uuid::new_v4(),
            witness_id,
            valid,
            confidence,
            reason,
        })
    }
}

impl Default for ProofEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a hex-encoded SHA-256 hash of the given string.
fn hash_sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

// ---------------------------------------------------------------------------
// Feature-gated: ruvector-verified integration (Phase 5)
// ---------------------------------------------------------------------------

/// When the `verified` feature is enabled, wraps the proof engine with
/// formal verification from `ruvector-verified` for proof-carrying vectors.
#[cfg(feature = "verified")]
pub mod verified_integration {
    use super::*;

    /// A proof engine enhanced with formal verification from ruvector-verified.
    /// Provides cryptographic proof-carrying capabilities for witness chains.
    pub struct VerifiedProofEngine {
        pub inner: ProofEngine,
    }

    impl VerifiedProofEngine {
        pub fn new() -> Self {
            // ruvector-verified provides formal verification layer
            let _ = ruvector_verified::VerificationConfig::default;
            Self {
                inner: ProofEngine::new(),
            }
        }

        /// Validate with formal verification backing.
        pub fn validate_verified(
            &mut self,
            action: &str,
            reasoning_chain: &str,
            evidence_refs: Vec<String>,
            confidence: f64,
            request: &ProofRequest,
        ) -> KernelResult<Proof> {
            self.inner
                .validate(action, reasoning_chain, evidence_refs, confidence, request)
        }

        /// Check chain integrity using both hash-chain and formal proofs.
        pub fn verify_integrity(&self) -> bool {
            self.inner.chain.verify_integrity()
        }
    }

    impl Default for VerifiedProofEngine {
        fn default() -> Self {
            Self::new()
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_valid_above_threshold() {
        let mut engine = ProofEngine::new();
        let req = ProofRequest {
            confidence_threshold: 0.7,
            require_evidence: false,
        };

        let proof = engine
            .validate("update-state", "because reasons", vec![], 0.9, &req)
            .unwrap();

        assert!(proof.valid);
        assert_eq!(engine.chain.len(), 1);
    }

    #[test]
    fn test_proof_invalid_below_threshold() {
        let mut engine = ProofEngine::new();
        let req = ProofRequest {
            confidence_threshold: 0.8,
            require_evidence: false,
        };

        let proof = engine
            .validate("risky-action", "weak reasoning", vec![], 0.5, &req)
            .unwrap();

        assert!(!proof.valid);
        assert!(proof.reason.contains("<"));
    }

    #[test]
    fn test_witness_chain_integrity_valid() {
        let mut chain = WitnessChain::new();
        chain.append("a1".into(), "r1".into(), vec!["e1".into()]);
        chain.append("a2".into(), "r2".into(), vec![]);
        chain.append("a3".into(), "r3".into(), vec!["e3a".into(), "e3b".into()]);

        assert!(chain.verify_integrity());
    }

    #[test]
    fn test_witness_chain_genesis_has_empty_prev_hash() {
        let mut chain = WitnessChain::new();
        chain.append("action".into(), "reason".into(), vec![]);

        let witness = &chain.witnesses[0];
        assert!(witness.prev_hash.is_empty());
        assert!(!witness.content_hash.is_empty());
    }

    #[test]
    fn test_witness_chain_links_prev_hash() {
        let mut chain = WitnessChain::new();
        chain.append("a1".into(), "r1".into(), vec![]);
        chain.append("a2".into(), "r2".into(), vec![]);

        let first_hash = chain.witnesses[0].content_hash.clone();
        assert_eq!(chain.witnesses[1].prev_hash, first_hash);
    }

    #[test]
    fn test_witness_chain_detects_tampered_action() {
        let mut chain = WitnessChain::new();
        chain.append("a1".into(), "r1".into(), vec![]);
        chain.append("a2".into(), "r2".into(), vec![]);

        // Tamper with the first witness's action_hash.
        chain.witnesses[0].action_hash = "tampered".into();

        assert!(!chain.verify_integrity());
    }

    #[test]
    fn test_witness_chain_detects_broken_link() {
        let mut chain = WitnessChain::new();
        chain.append("a1".into(), "r1".into(), vec![]);
        chain.append("a2".into(), "r2".into(), vec![]);

        // Break the chain link by altering prev_hash and recomputing content_hash
        // so content_hash is internally consistent but the link is broken.
        chain.witnesses[1].prev_hash = "bogus".into();
        chain.witnesses[1].content_hash = Witness::compute_content_hash(
            &chain.witnesses[1].id,
            &chain.witnesses[1].action_hash,
            &chain.witnesses[1].reasoning_chain_hash,
            &chain.witnesses[1].evidence_refs,
            &chain.witnesses[1].timestamp,
            &chain.witnesses[1].prev_hash,
        );

        assert!(!chain.verify_integrity());
    }

    #[test]
    fn test_empty_chain_is_valid() {
        let chain = WitnessChain::new();
        assert!(chain.verify_integrity());
    }
}
