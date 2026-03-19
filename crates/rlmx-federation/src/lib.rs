//! # rlmx-federation
//!
//! Federated learning pipeline for the RuVix cognition kernel (ADR-023, DDD-012).
//!
//! Implements weekly federation cycles with on-device anonymization, cloud-side
//! aggregation, and bidirectional LoRA distribution. Core privacy invariants:
//!
//! - No raw user data ever leaves the device.
//! - Patterns anonymized BEFORE leaving device (PII stripped, emotion bucketed,
//!   Laplace noise applied).
//! - Pseudonymous contribution keys (not linkable to user identity).
//! - Minimum 1000-user aggregation threshold before any pattern is published.
//! - No speaker embeddings federated.
//!
//! ## Bounded Context
//!
//! **Aggregate root**: [`FederationCycle`](cycle::FederationCycle)
//!
//! A cycle progresses through four statuses:
//! `Collecting` -> `Aggregating` -> `Distributing` -> `Completed`
//!
//! ## Crate Layout
//!
//! - [`cycle`] — FederationCycle aggregate root
//! - [`anonymizer`] — On-device anonymization pipeline
//! - [`contribution`] — User contributions (pseudonymous, domain-scoped)
//! - [`aggregator`] — Cloud-side pattern aggregation
//! - [`distribution`] — Package building and distribution
//! - [`bootstrap`] — New user bootstrap from federated patterns
//! - [`error`] — Federation error types

pub mod aggregator;
pub mod anonymizer;
pub mod bootstrap;
pub mod contribution;
pub mod cycle;
pub mod distribution;
pub mod error;

// Re-export primary types for convenient access.
pub use aggregator::{AggregatedModel, AggregatorConfig, FederatedAggregator};
pub use anonymizer::{AnonymizationConfig, FederationAnonymizer};
pub use bootstrap::{bootstrap_from_latest, bootstrap_from_package, BootstrapResult};
pub use contribution::{generate_pseudonym, Contribution};
pub use cycle::{CycleStatus, FederationCycle};
pub use distribution::{FederationPackage, PackageDistributor};
pub use error::{FederationError, Result};
