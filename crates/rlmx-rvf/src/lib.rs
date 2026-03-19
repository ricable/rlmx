//! rlmx-rvf: RVF (RuVix Format) container packaging and security layer.
//!
//! This crate provides the core container format for packaging plugins, models,
//! and configurations into sealed, cryptographically verifiable units. It
//! includes:
//!
//! - **Container** — the top-level RVF packaging unit with manifest, segments,
//!   witness chain, and optional signature.
//! - **Segments** — 21+ typed data segments (vectors, indexes, WASM, eBPF, etc.)
//! - **Witness Chain** — append-only cryptographic audit trail with hash chaining
//!   and Ed25519 signatures.
//! - **Crypto** — Ed25519 signing/verification and SHA-256 hashing (with a
//!   placeholder for post-quantum ML-DSA-65).
//! - **RBAC** — six-role access control model per the PRD.
//! - **Branch** — copy-on-write branching and merging of containers.

pub mod branch;
pub mod container;
pub mod crypto;
pub mod rbac;
pub mod segments;
pub mod witness;

// Re-export primary types at the crate root for convenience.
pub use branch::{BranchManager, RvfBranch};
pub use container::{RvfContainer, RvfManifest};
pub use crypto::{hash_sha256, ContainerSignature, CryptoEngine};
pub use rbac::{AccessControl, AccessPolicy, Operation, Role};
pub use segments::{DiffKind, RvfSegment, SegmentDiff, SegmentType};
pub use witness::{WitnessChain, WitnessEntry};

#[cfg(feature = "rvf-ext")]
pub mod rvf_ext {
    use tracing::debug;
    pub mod types {
        pub use rvf_types::*;
    }
    pub mod runtime {
        pub use rvf_runtime::*;
    }
    pub mod wire {
        pub use rvf_wire::*;
    }
    pub mod index {
        pub use rvf_index::*;
    }
    pub mod quant {
        pub use rvf_quant::*;
    }
    pub mod crypto_ext {
        pub use rvf_crypto::*;
    }
}

/// Errors that can occur during RVF operations.
#[derive(Debug, thiserror::Error)]
pub enum RvfError {
    /// Serialization or deserialization failure.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// File I/O failure.
    #[error("I/O error: {0}")]
    Io(String),

    /// Cryptographic verification failure.
    #[error("crypto error: {0}")]
    Crypto(String),

    /// Witness chain integrity failure.
    #[error("witness chain error: {0}")]
    WitnessChain(String),

    /// Branch management error.
    #[error("branch error: {0}")]
    Branch(String),

    /// Access control violation.
    #[error("access denied: {0}")]
    AccessDenied(String),
}
