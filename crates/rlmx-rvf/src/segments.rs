//! RVF Segment types (PRD Section 2.3)
//!
//! Defines all 24+ segment types that can be packaged inside an RVF container.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// All segment types supported by the RVF container format.
///
/// Each segment type corresponds to a distinct category of data that can be
/// packaged, versioned, and cryptographically verified within an RVF container.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentType {
    /// Vector embeddings
    Vec,
    /// HNSW index data
    Index,
    /// Root manifest
    Manifest,
    /// Quantization metadata
    Quant,
    /// Audit trail witness
    Witness,
    /// Cryptographic material
    Crypto,
    /// Kernel state
    Kernel,
    /// eBPF programs
    Ebpf,
    /// WASM modules
    Wasm,
    /// Copy-on-write mapping
    CowMap,
    /// Cluster membership
    Membership,
    /// Delta for federation sync
    Delta,
    /// Transfer learning priors
    TransferPrior,
    /// Policy kernel data
    PolicyKernel,
    /// Cost optimization curves
    CostCurve,
    /// LoRA overlay weights
    Overlay,
    /// Entity graph state
    Graph,
    /// Probabilistic sketches
    Sketch,
    /// Configuration
    Config,
    /// Model weights
    Model,
    /// SONA pattern bank
    Pattern,
    /// User-defined custom segment type
    Custom(String),
}

impl std::fmt::Display for SegmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SegmentType::Vec => write!(f, "Vec"),
            SegmentType::Index => write!(f, "Index"),
            SegmentType::Manifest => write!(f, "Manifest"),
            SegmentType::Quant => write!(f, "Quant"),
            SegmentType::Witness => write!(f, "Witness"),
            SegmentType::Crypto => write!(f, "Crypto"),
            SegmentType::Kernel => write!(f, "Kernel"),
            SegmentType::Ebpf => write!(f, "Ebpf"),
            SegmentType::Wasm => write!(f, "Wasm"),
            SegmentType::CowMap => write!(f, "CowMap"),
            SegmentType::Membership => write!(f, "Membership"),
            SegmentType::Delta => write!(f, "Delta"),
            SegmentType::TransferPrior => write!(f, "TransferPrior"),
            SegmentType::PolicyKernel => write!(f, "PolicyKernel"),
            SegmentType::CostCurve => write!(f, "CostCurve"),
            SegmentType::Overlay => write!(f, "Overlay"),
            SegmentType::Graph => write!(f, "Graph"),
            SegmentType::Sketch => write!(f, "Sketch"),
            SegmentType::Config => write!(f, "Config"),
            SegmentType::Model => write!(f, "Model"),
            SegmentType::Pattern => write!(f, "Pattern"),
            SegmentType::Custom(name) => write!(f, "Custom({})", name),
        }
    }
}

/// A single segment within an RVF container.
///
/// Each segment carries typed binary data along with metadata and a SHA-256
/// integrity hash computed over the data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RvfSegment {
    /// Unique identifier for this segment.
    pub id: Uuid,
    /// The type of data this segment carries.
    pub segment_type: SegmentType,
    /// Raw binary data payload.
    pub data: Vec<u8>,
    /// Arbitrary JSON metadata associated with this segment.
    pub metadata: serde_json::Value,
    /// SHA-256 hex digest of `data`.
    pub hash: String,
}

impl RvfSegment {
    /// Create a new segment, automatically computing the SHA-256 hash of the data.
    pub fn new(
        segment_type: SegmentType,
        data: Vec<u8>,
        metadata: serde_json::Value,
    ) -> Self {
        let hash = crate::crypto::hash_sha256(&data);
        Self {
            id: Uuid::new_v4(),
            segment_type,
            data,
            metadata,
            hash,
        }
    }

    /// Verify that the stored hash matches the actual data.
    pub fn verify_integrity(&self) -> bool {
        let computed = crate::crypto::hash_sha256(&self.data);
        computed == self.hash
    }
}


/// Represents a difference between two segments (used by branch diffing).
#[derive(Debug, Clone)]
pub struct SegmentDiff {
    /// The segment ID.
    pub segment_id: Uuid,
    /// The kind of change.
    pub kind: DiffKind,
}

/// Kind of segment difference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffKind {
    /// Segment was added in the branch.
    Added,
    /// Segment was modified in the branch.
    Modified,
    /// Segment was removed in the branch.
    Removed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_type_serialization_roundtrip() {
        let types = vec![
            SegmentType::Vec,
            SegmentType::Index,
            SegmentType::Manifest,
            SegmentType::Quant,
            SegmentType::Witness,
            SegmentType::Crypto,
            SegmentType::Kernel,
            SegmentType::Ebpf,
            SegmentType::Wasm,
            SegmentType::CowMap,
            SegmentType::Membership,
            SegmentType::Delta,
            SegmentType::TransferPrior,
            SegmentType::PolicyKernel,
            SegmentType::CostCurve,
            SegmentType::Overlay,
            SegmentType::Graph,
            SegmentType::Sketch,
            SegmentType::Config,
            SegmentType::Model,
            SegmentType::Pattern,
            SegmentType::Custom("my_type".to_string()),
        ];

        for seg_type in &types {
            let json = serde_json::to_string(seg_type).expect("serialize");
            let deser: SegmentType = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(*seg_type, deser, "roundtrip failed for {:?}", seg_type);
        }
    }

    #[test]
    fn test_segment_integrity_verification() {
        let segment = RvfSegment::new(
            SegmentType::Vec,
            vec![1, 2, 3, 4, 5],
            serde_json::json!({"dim": 128}),
        );
        assert!(segment.verify_integrity());
    }
}
