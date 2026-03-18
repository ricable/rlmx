//! # rlmx-trm
//!
//! Tiny Recursive Model (TRM) scheduling policy for the RuVix kernel.
//!
//! Implements a 2-layer neural network that recursively refines predictions
//! through three streams (x, y, z) with adaptive halting.

pub mod config;
pub mod halting;
pub mod model;
pub mod strategy;
pub mod streams;

// Re-exports for convenient access.
pub use config::{HaltConfig, TrmClassification, TrmConfig, TrmInput, TrmModelConfig, TrmResult};
pub use halting::{AdaptiveHalter, HaltCondition};
pub use model::TrmModel;
pub use strategy::TrmStrategy;
pub use streams::TrmStreams;
