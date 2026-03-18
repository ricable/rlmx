//! RVF Container — the top-level packaging unit.
//!
//! An `RvfContainer` bundles a manifest, typed segments, a witness chain, and
//! an optional cryptographic signature into a single serializable artifact.

use std::path::Path;

use chrono::{DateTime, Utc};
use ed25519_dalek::{SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::crypto::{hash_sha256, ContainerSignature, verify_with_key};
use crate::segments::{RvfSegment, SegmentType};
use crate::witness::WitnessChain;
use crate::RvfError;

/// The root manifest describing an RVF container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RvfManifest {
    /// Unique container identifier.
    pub id: Uuid,
    /// Semantic version string.
    pub version: String,
    /// Timestamp when the container was created.
    pub created: DateTime<Utc>,
    /// Name of the plugin this container belongs to.
    pub plugin: String,
    /// Number of segments in the container.
    pub segment_count: usize,
    /// Total size of all segment data in bytes.
    pub total_size_bytes: u64,
    /// Arbitrary configuration carried with the container.
    pub config: serde_json::Value,
}

/// A complete RVF container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RvfContainer {
    /// The container manifest.
    pub manifest: RvfManifest,
    /// All segments packaged in this container.
    pub segments: Vec<RvfSegment>,
    /// The cryptographic audit trail.
    pub witness_chain: WitnessChain,
    /// Optional Ed25519 signature over the container contents.
    pub signature: Option<ContainerSignature>,
}

impl RvfContainer {
    /// Create a new, unsigned container for the given plugin.
    pub fn new(plugin: &str, config: serde_json::Value) -> Self {
        Self {
            manifest: RvfManifest {
                id: Uuid::new_v4(),
                version: "0.1.0".to_string(),
                created: Utc::now(),
                plugin: plugin.to_string(),
                segment_count: 0,
                total_size_bytes: 0,
                config,
            },
            segments: Vec::new(),
            witness_chain: WitnessChain::new(),
            signature: None,
        }
    }

    /// Add a new segment and return its UUID.
    pub fn add_segment(
        &mut self,
        segment_type: SegmentType,
        data: Vec<u8>,
        metadata: serde_json::Value,
    ) -> Uuid {
        let segment = RvfSegment::new(segment_type, data, metadata);
        let id = segment.id;
        self.manifest.total_size_bytes += segment.data.len() as u64;
        self.segments.push(segment);
        self.manifest.segment_count = self.segments.len();
        id
    }

    /// Seal (sign) the container using the provided signing key.
    ///
    /// After sealing, the container carries a `ContainerSignature` that covers
    /// the canonical content hash of the manifest and segments.
    pub fn seal(&mut self, signing_key: &SigningKey) -> Result<(), RvfError> {
        let content_hash = self.content_hash();

        use ed25519_dalek::Signer;
        let sig = signing_key.sign(content_hash.as_bytes());
        let vk = signing_key.verifying_key();

        self.signature = Some(ContainerSignature {
            content_hash,
            signature: sig.to_bytes().to_vec(),
            verifying_key_bytes: vk.to_bytes().to_vec(),
        });

        Ok(())
    }

    /// Verify the container signature against the provided verifying key.
    ///
    /// This also checks that the provided key matches the key embedded in the
    /// signature, ensuring the signature is bound to the expected signer.
    pub fn verify(&self, verifying_key: &VerifyingKey) -> Result<bool, RvfError> {
        let Some(ref sig) = self.signature else {
            return Ok(false);
        };

        // Verify that the provided key matches the key stored in the signature.
        if verifying_key.to_bytes().to_vec() != sig.verifying_key_bytes {
            return Err(RvfError::Crypto(
                "verifying key does not match the key embedded in the container signature"
                    .to_string(),
            ));
        }

        let content_hash = self.content_hash();
        if content_hash != sig.content_hash {
            return Ok(false);
        }

        Ok(verify_with_key(
            verifying_key,
            content_hash.as_bytes(),
            &sig.signature,
        ))
    }

    /// Save the container to disk as JSON.
    pub fn save(&self, path: &Path) -> Result<(), RvfError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| RvfError::Serialization(e.to_string()))?;
        std::fs::write(path, json).map_err(|e| RvfError::Io(e.to_string()))?;
        Ok(())
    }

    /// Load a container from a JSON file on disk.
    pub fn load(path: &Path) -> Result<Self, RvfError> {
        let data = std::fs::read_to_string(path).map_err(|e| RvfError::Io(e.to_string()))?;
        let container: RvfContainer =
            serde_json::from_str(&data).map_err(|e| RvfError::Serialization(e.to_string()))?;
        Ok(container)
    }

    /// Create a copy-on-write branch (fork) of this container.
    pub fn branch(&self, name: &str) -> Result<RvfContainer, RvfError> {
        let mut branched = self.clone();
        branched.manifest.id = Uuid::new_v4();
        branched.manifest.version = format!("{}-branch-{}", self.manifest.version, name);
        branched.signature = None; // branches are unsigned until re-sealed
        Ok(branched)
    }

    /// Compute the canonical content hash of the manifest + segments.
    fn content_hash(&self) -> String {
        let manifest_json =
            serde_json::to_string(&self.manifest).unwrap_or_default();
        let segments_json =
            serde_json::to_string(&self.segments).unwrap_or_default();
        let combined = format!("{}{}", manifest_json, segments_json);
        hash_sha256(combined.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segments::SegmentType;
    use rand::rngs::OsRng;

    fn make_test_container() -> RvfContainer {
        let mut c = RvfContainer::new("test-plugin", serde_json::json!({"k": "v"}));
        c.add_segment(
            SegmentType::Vec,
            vec![10, 20, 30],
            serde_json::json!({"dim": 3}),
        );
        c.add_segment(
            SegmentType::Config,
            b"config data".to_vec(),
            serde_json::json!({}),
        );
        c
    }

    #[test]
    fn test_container_create_and_add_segment() {
        let c = make_test_container();
        assert_eq!(c.segments.len(), 2);
        assert_eq!(c.manifest.segment_count, 2);
        assert_eq!(c.manifest.total_size_bytes, 3 + 11);
    }

    #[test]
    fn test_container_seal_and_verify() {
        let mut c = make_test_container();
        let sk = SigningKey::generate(&mut OsRng);
        let vk = sk.verifying_key();

        c.seal(&sk).unwrap();
        assert!(c.signature.is_some());
        assert!(c.verify(&vk).unwrap());

        // Verify with wrong key should return a crypto error (key binding check).
        let other_sk = SigningKey::generate(&mut OsRng);
        let other_vk = other_sk.verifying_key();
        assert!(c.verify(&other_vk).is_err());
    }

    #[test]
    fn test_container_save_and_load_roundtrip() {
        let mut c = make_test_container();
        let sk = SigningKey::generate(&mut OsRng);
        c.seal(&sk).unwrap();

        let dir = std::env::temp_dir().join(format!("rvf_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_container.rvf.json");

        c.save(&path).unwrap();
        let loaded = RvfContainer::load(&path).unwrap();

        assert_eq!(loaded.manifest.plugin, "test-plugin");
        assert_eq!(loaded.segments.len(), 2);
        assert!(loaded.signature.is_some());

        // Verify the loaded container's signature.
        let vk = sk.verifying_key();
        assert!(loaded.verify(&vk).unwrap());

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}
