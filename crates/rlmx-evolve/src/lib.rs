//! rlmx-evolve — Dynamic function evolution for RLMX agents.
//!
//! Provides a lifecycle state machine, scoring engine, feedback loop,
//! and DAG-based version tracking for evolved functions.

pub mod types;
pub mod lifecycle;
pub mod scorer;
pub mod feedback;
pub mod dag;

pub use types::*;
pub use lifecycle::LifecycleManager;
pub use scorer::ScoreEngine;
pub use feedback::FeedbackEngine;
pub use dag::FunctionDag;
