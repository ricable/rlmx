use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
    /// Placeholder for Ed25519 signature.
    pub signature: String,
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
        let witness = Witness {
            id,
            action_hash,
            reasoning_chain_hash,
            evidence_refs,
            timestamp: Utc::now(),
            signature: "placeholder-ed25519".into(),
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

        let action_hash = simple_hash(action);
        let reasoning_hash = simple_hash(reasoning_chain);

        let witness_id = self.chain.append(action_hash, reasoning_hash, evidence_refs);

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

/// Trivial hash placeholder (not cryptographic).
fn simple_hash(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
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
}
