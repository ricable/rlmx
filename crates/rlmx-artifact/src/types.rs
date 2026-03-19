use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Content-addressed artifact identifier (SHA-256 hash).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactId(pub [u8; 32]);

impl ArtifactId {
    /// Create an ArtifactId from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Return the raw bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Create from a hex string (64 characters).
    pub fn from_hex(hex_str: &str) -> Result<Self, ArtifactError> {
        let bytes = hex::decode(hex_str).map_err(|e| {
            ArtifactError::InvalidParent(format!("invalid hex: {e}"))
        })?;
        if bytes.len() != 32 {
            return Err(ArtifactError::InvalidParent(format!(
                "hex must decode to 32 bytes, got {}",
                bytes.len()
            )));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }

    /// Return the hex representation.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Display for ArtifactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for ArtifactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArtifactId({})", &self.to_hex()[..12])
    }
}

/// A content-addressed artifact in the DAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: ArtifactId,
    pub content: Vec<u8>,
    pub content_type: String,
    pub parent_ids: Vec<ArtifactId>,
    pub creator: u64,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

/// Diff between two artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactDiff {
    pub old_id: ArtifactId,
    pub new_id: ArtifactId,
    pub content_type: String,
    pub added_bytes: usize,
    pub removed_bytes: usize,
}

/// Errors for artifact operations.
#[derive(Debug, thiserror::Error)]
pub enum ArtifactError {
    #[error("artifact not found: {0}")]
    NotFound(String),

    #[error("duplicate artifact id: {0}")]
    DuplicateId(ArtifactId),

    #[error("invalid parent: {0}")]
    InvalidParent(String),

    #[error("branch conflict: {0}")]
    BranchConflict(String),

    #[error("store error: {0}")]
    StoreError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_id_display() {
        let id = ArtifactId([0xab; 32]);
        let hex = id.to_string();
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c == 'a' || c == 'b'));
    }

    #[test]
    fn test_artifact_id_debug() {
        let id = ArtifactId([0xcd; 32]);
        let dbg = format!("{id:?}");
        assert!(dbg.starts_with("ArtifactId("));
        assert!(dbg.contains("cdcdcdcdcdcd"));
    }

    #[test]
    fn test_artifact_id_eq() {
        let a = ArtifactId([1; 32]);
        let b = ArtifactId([1; 32]);
        let c = ArtifactId([2; 32]);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_artifact_id_hash() {
        use std::collections::HashSet;
        let a = ArtifactId([1; 32]);
        let b = ArtifactId([1; 32]);
        let mut set = HashSet::new();
        set.insert(a);
        set.insert(b);
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_artifact_id_from_hex() {
        let hex = "ab".repeat(32);
        let id = ArtifactId::from_hex(&hex).unwrap();
        assert_eq!(id.0, [0xab; 32]);
    }

    #[test]
    fn test_artifact_id_from_hex_invalid_length() {
        let result = ArtifactId::from_hex("abcd");
        assert!(result.is_err());
    }

    #[test]
    fn test_artifact_id_from_hex_invalid_chars() {
        let hex = "zz".repeat(32);
        let result = ArtifactId::from_hex(&hex);
        assert!(result.is_err());
    }

    #[test]
    fn test_artifact_id_roundtrip_hex() {
        let original = ArtifactId([0x42; 32]);
        let hex = original.to_hex();
        let recovered = ArtifactId::from_hex(&hex).unwrap();
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_artifact_id_serialize() {
        let id = ArtifactId([0x01; 32]);
        let json = serde_json::to_string(&id).unwrap();
        let recovered: ArtifactId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, recovered);
    }

    #[test]
    fn test_artifact_serialize() {
        let artifact = Artifact {
            id: ArtifactId([0; 32]),
            content: b"hello".to_vec(),
            content_type: "text/plain".into(),
            parent_ids: vec![],
            creator: 42,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&artifact).unwrap();
        assert!(json.contains("text/plain"));
    }

    #[test]
    fn test_artifact_error_display() {
        let err = ArtifactError::NotFound("abc".into());
        assert_eq!(err.to_string(), "artifact not found: abc");
    }

    #[test]
    fn test_artifact_error_duplicate() {
        let id = ArtifactId([0xff; 32]);
        let err = ArtifactError::DuplicateId(id);
        assert!(err.to_string().contains("duplicate artifact id"));
    }

    #[test]
    fn test_artifact_diff_fields() {
        let diff = ArtifactDiff {
            old_id: ArtifactId([1; 32]),
            new_id: ArtifactId([2; 32]),
            content_type: "text/plain".into(),
            added_bytes: 100,
            removed_bytes: 50,
        };
        assert_eq!(diff.added_bytes, 100);
        assert_eq!(diff.removed_bytes, 50);
    }
}
