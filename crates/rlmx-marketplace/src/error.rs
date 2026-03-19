//! Marketplace error types.

use uuid::Uuid;

use crate::domain::{PublisherId, ReviewStatus};

/// Errors produced by the marketplace bounded context.
#[derive(Debug, thiserror::Error)]
pub enum MarketplaceError {
    #[error("listing not found: {0}")]
    ListingNotFound(Uuid),

    #[error("publisher not found: {0:?}")]
    PublisherNotFound(PublisherId),

    #[error("submission not found: {0}")]
    SubmissionNotFound(Uuid),

    #[error("duplicate email: {0}")]
    DuplicateEmail(String),

    #[error("invalid review state for submission {0}: {1:?}")]
    InvalidReviewState(Uuid, ReviewStatus),

    #[error("agent not approved for publishing: {0}")]
    NotApproved(Uuid),

    #[error("invalid revenue split: percentages must sum to 100")]
    InvalidRevenueSplit,

    #[error("publisher not verified: {0:?}")]
    PublisherNotVerified(PublisherId),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type MarketplaceResult<T> = Result<T, MarketplaceError>;
