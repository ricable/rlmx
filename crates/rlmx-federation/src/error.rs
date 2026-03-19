//! Federation-specific error types.

use thiserror::Error;
use uuid::Uuid;

use rlmx_kernel::LifeDomain;

/// Errors that can occur during federated learning operations.
#[derive(Debug, Error)]
pub enum FederationError {
    #[error("cycle {0} is not in the expected status for this operation")]
    InvalidCycleStatus(Uuid),

    #[error("aggregation threshold not met: need {required} contributors, have {actual}")]
    AggregationThresholdNotMet { required: usize, actual: usize },

    #[error("contribution rejected: {reason}")]
    ContributionRejected { reason: String },

    #[error("no contributions for domain {0:?} in this cycle")]
    NoDomainContributions(LifeDomain),

    #[error("package not found: {0}")]
    PackageNotFound(Uuid),

    #[error("cycle not found: {0}")]
    CycleNotFound(Uuid),

    #[error("anonymization failed: {0}")]
    AnonymizationFailed(String),

    #[error("distribution failed: {0}")]
    DistributionFailed(String),

    #[error("bootstrap failed: {0}")]
    BootstrapFailed(String),

    #[error("privacy invariant violated: {0}")]
    PrivacyViolation(String),
}

pub type Result<T> = std::result::Result<T, FederationError>;
