//! Witness Chain — cryptographic audit trail for RVF containers.
//!
//! Each entry in the chain records an action, its reasoning hash, evidence
//! references, and a signature. Entries form a hash chain (each entry includes
//! the hash of the previous entry) so tampering is detectable.

use chrono::{DateTime, Utc};
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::{hash_sha256, verify_with_key};

/// An append-only chain of witnessed actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessChain {
    /// Ordered list of witness entries forming the hash chain.
    pub entries: Vec<WitnessEntry>,
}

/// A single entry in the witness chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessEntry {
    /// Unique identifier for this entry.
    pub id: Uuid,
    /// Timestamp of when the action was recorded.
    pub timestamp: DateTime<Utc>,
    /// Human-readable description of the action.
    pub action: String,
    /// SHA-256 hash of the action and its parameters.
    pub action_hash: String,
    /// SHA-256 hash of the reasoning chain that led to this action.
    pub reasoning_chain_hash: String,
    /// UUIDs of segments referenced as evidence.
    pub evidence_refs: Vec<Uuid>,
    /// Identifier of the agent that performed the action.
    pub agent_id: Uuid,
    /// Confidence score in [0.0, 1.0].
    pub confidence: f64,
    /// Ed25519 signature over the canonical entry bytes.
    pub signature: Vec<u8>,
    /// Hash of the previous entry (empty string for the first entry).
    pub prev_hash: String,
}

impl WitnessChain {
    /// Create an empty witness chain.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Append a new entry to the chain, signing it with the provided key.
    ///
    /// Returns a reference to the newly appended entry.
    pub fn append(
        &mut self,
        action: &str,
        reasoning: &str,
        evidence_refs: Vec<Uuid>,
        agent_id: Uuid,
        confidence: f64,
        signing_key: &SigningKey,
    ) -> &WitnessEntry {
        let prev_hash = if let Some(last) = self.entries.last() {
            hash_entry(last)
        } else {
            String::new()
        };

        let id = Uuid::new_v4();
        let timestamp = Utc::now();
        let action_hash = hash_sha256(action.as_bytes());
        let reasoning_chain_hash = hash_sha256(reasoning.as_bytes());

        // Build canonical bytes for signing (all fields).
        let sign_payload = canonical_payload(
            &id,
            &timestamp,
            &agent_id,
            confidence,
            &evidence_refs,
            &action_hash,
            &reasoning_chain_hash,
            &prev_hash,
        );
        let sig = {
            use ed25519_dalek::Signer;
            signing_key.sign(&sign_payload).to_bytes().to_vec()
        };

        let entry = WitnessEntry {
            id,
            timestamp,
            action: action.to_string(),
            action_hash,
            reasoning_chain_hash,
            evidence_refs,
            agent_id,
            confidence,
            signature: sig,
            prev_hash,
        };

        self.entries.push(entry);
        self.entries.last().unwrap()
    }

    /// Verify the hash-chain integrity (each entry's `prev_hash` matches).
    pub fn verify_integrity(&self) -> Result<bool, crate::RvfError> {
        for (i, entry) in self.entries.iter().enumerate() {
            if i == 0 {
                if !entry.prev_hash.is_empty() {
                    return Ok(false);
                }
            } else {
                let expected = hash_entry(&self.entries[i - 1]);
                if entry.prev_hash != expected {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    /// Verify every entry's Ed25519 signature against the given verifying key.
    pub fn verify_signatures(
        &self,
        verifying_key: &ed25519_dalek::VerifyingKey,
    ) -> Result<bool, crate::RvfError> {
        for entry in &self.entries {
            let payload = canonical_payload(
                &entry.id,
                &entry.timestamp,
                &entry.agent_id,
                entry.confidence,
                &entry.evidence_refs,
                &entry.action_hash,
                &entry.reasoning_chain_hash,
                &entry.prev_hash,
            );
            if !verify_with_key(verifying_key, &payload, &entry.signature) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Number of entries in the chain.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Look up an entry by its UUID.
    pub fn get(&self, id: Uuid) -> Option<&WitnessEntry> {
        self.entries.iter().find(|e| e.id == id)
    }
}

impl Default for WitnessChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a canonical byte payload for signing / verifying.
///
/// Covers ALL fields so that tampering with any field is detectable.
fn canonical_payload(
    id: &Uuid,
    timestamp: &DateTime<Utc>,
    agent_id: &Uuid,
    confidence: f64,
    evidence_refs: &[Uuid],
    action_hash: &str,
    reasoning_hash: &str,
    prev_hash: &str,
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(id.to_string().as_bytes());
    buf.extend_from_slice(timestamp.to_rfc3339().as_bytes());
    buf.extend_from_slice(agent_id.to_string().as_bytes());
    buf.extend_from_slice(confidence.to_bits().to_le_bytes().as_slice());
    for r in evidence_refs {
        buf.extend_from_slice(r.to_string().as_bytes());
    }
    buf.extend_from_slice(action_hash.as_bytes());
    buf.extend_from_slice(reasoning_hash.as_bytes());
    buf.extend_from_slice(prev_hash.as_bytes());
    buf
}

/// Hash an entire entry to produce the chain link.
///
/// Includes all fields so that tampering with any field breaks the chain.
fn hash_entry(entry: &WitnessEntry) -> String {
    let evidence_str: String = entry
        .evidence_refs
        .iter()
        .map(|r| r.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let canonical = format!(
        "{}:{}:{}:{}:{}:{}:{}:{}",
        entry.id,
        entry.timestamp.to_rfc3339(),
        entry.agent_id,
        entry.confidence,
        evidence_str,
        entry.action_hash,
        entry.reasoning_chain_hash,
        entry.prev_hash,
    );
    hash_sha256(canonical.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;

    fn test_signing_key() -> SigningKey {
        SigningKey::generate(&mut OsRng)
    }

    #[test]
    fn test_witness_chain_append_and_len() {
        let sk = test_signing_key();
        let mut chain = WitnessChain::new();
        assert_eq!(chain.len(), 0);

        let agent = Uuid::new_v4();
        chain.append("ingest", "needed data", vec![], agent, 0.95, &sk);
        chain.append("query", "user asked", vec![], agent, 0.9, &sk);

        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_witness_chain_verify_integrity() {
        let sk = test_signing_key();
        let mut chain = WitnessChain::new();
        let agent = Uuid::new_v4();

        chain.append("action1", "reason1", vec![], agent, 1.0, &sk);
        chain.append("action2", "reason2", vec![], agent, 0.8, &sk);
        chain.append("action3", "reason3", vec![], agent, 0.7, &sk);

        assert!(chain.verify_integrity().unwrap());
    }

    #[test]
    fn test_witness_chain_verify_signatures() {
        let sk = test_signing_key();
        let vk = sk.verifying_key();
        let mut chain = WitnessChain::new();
        let agent = Uuid::new_v4();

        chain.append("a1", "r1", vec![], agent, 1.0, &sk);
        chain.append("a2", "r2", vec![], agent, 0.9, &sk);

        assert!(chain.verify_signatures(&vk).unwrap());

        // A different key should fail verification.
        let other = test_signing_key();
        let other_vk = other.verifying_key();
        assert!(!chain.verify_signatures(&other_vk).unwrap());
    }
}
