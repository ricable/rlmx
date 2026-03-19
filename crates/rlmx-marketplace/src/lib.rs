//! # rlmx-marketplace
//!
//! Agent marketplace bounded context (DDD-010, ADR-014).
//! Manages publishing, discovery, installation, billing, review, and analytics
//! for the RLMX agent ecosystem.

pub mod analytics;
pub mod billing;
pub mod domain;
pub mod error;
pub mod featured;
pub mod marketplace;
pub mod publisher;
pub mod registry;
pub mod review;

// Re-export primary types for convenient access.
pub use analytics::MarketplaceAnalytics;
pub use billing::BillingEngine;
pub use domain::{
    AgentPack, AgentPrice, DeveloperType, DeviceType, LifeDomain, ListingStatus,
    MarketplaceDomainEvent, ModelTier, PayoutMethod, Permission, PublisherId, RevenueSplit,
    ReviewStatus, SecurityCheck, SecurityCheckType, Severity,
};
pub use error::{MarketplaceError, MarketplaceResult};
pub use featured::{FeaturedEngine, ScoredAgent};
pub use marketplace::Marketplace;
pub use publisher::{Publisher, PublisherPortal};
pub use registry::{AgentListing, AgentRegistry, ListingFilter, ListingSort};
pub use review::{ReviewDecision, ReviewPipeline, ReviewSubmission};
