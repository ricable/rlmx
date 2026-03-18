//! Cryptographic operations for RVF containers.
//!
//! Provides Ed25519 signing/verification and SHA-256 hashing utilities.
//! Includes a placeholder for future ML-DSA-65 post-quantum support.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Wrapper around Ed25519 key material that provides signing and verification.
pub struct CryptoEngine {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl CryptoEngine {
    /// Generate a new random Ed25519 keypair.
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Create a `CryptoEngine` from an existing signing key.
    pub fn from_signing_key(signing_key: SigningKey) -> Self {
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Return a reference to the signing key.
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    /// Return a reference to the verifying key.
    pub fn verifying_key(&self) -> &VerifyingKey {
        &self.verifying_key
    }

    /// Sign arbitrary data, returning the raw signature bytes.
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let sig = self.signing_key.sign(data);
        sig.to_bytes().to_vec()
    }

    /// Verify a signature against data. Returns `true` if valid.
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        verify_with_key(&self.verifying_key, data, signature)
    }
}

/// Verify a signature using the provided verifying key.
pub fn verify_with_key(verifying_key: &VerifyingKey, data: &[u8], signature: &[u8]) -> bool {
    if signature.len() != 64 {
        return false;
    }
    let mut sig_bytes = [0u8; 64];
    sig_bytes.copy_from_slice(signature);
    match Signature::from_bytes(&sig_bytes) {
        sig => verifying_key.verify(data, &sig).is_ok(),
    }
}

/// Compute a hex-encoded SHA-256 hash of the provided data.
pub fn hash_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Container signature stored alongside an RVF container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSignature {
    /// The hex-encoded SHA-256 hash of the serialized manifest + segments.
    pub content_hash: String,
    /// The raw Ed25519 signature bytes.
    pub signature: Vec<u8>,
    /// The verifying (public) key bytes used to validate.
    pub verifying_key_bytes: Vec<u8>,
}

// ──────────────────────────────────────────────────────────────────────────────
// Post-quantum placeholder: ML-DSA-65 (FIPS 204)
//
// When a mature Rust implementation of ML-DSA-65 becomes available, we will add
// a `PostQuantumEngine` here that mirrors the `CryptoEngine` API. The RVF
// container format already stores raw signature bytes, so swapping algorithms
// will require only an engine change plus a version bump in the manifest.
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_engine_sign_and_verify() {
        let engine = CryptoEngine::generate();
        let data = b"hello rvf world";
        let sig = engine.sign(data);
        assert!(engine.verify(data, &sig));
    }

    #[test]
    fn test_crypto_engine_verify_wrong_data() {
        let engine = CryptoEngine::generate();
        let data = b"correct data";
        let sig = engine.sign(data);
        assert!(!engine.verify(b"wrong data", &sig));
    }

    #[test]
    fn test_hash_sha256_known_value() {
        // SHA-256 of empty input is a well-known constant.
        let hash = hash_sha256(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
