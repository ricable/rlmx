//! rlmx-cognitive: Self-learning and cognitive layer for the RuVix kernel.
//!
//! This crate implements the cognitive self-improvement layer including:
//! - **SONA**: Self-Optimizing Neural Architecture (micro-LoRA, pattern bank)
//! - **DAG Optimizer**: Self-learning query execution strategy optimizer
//! - **Nervous System**: Bio-inspired components (BTSP, HDC, WTA, Circadian, Global Workspace)
//! - **Attention Selector**: Auto-selection among 39 attention mechanisms

pub mod attention;
pub mod dag;
pub mod nervous;
pub mod sona;

// Re-export primary types for convenience.
pub use attention::{AttentionCategory, AttentionMechanism, AttentionSelector};
pub use dag::{
    DagOptimizer, ExecutionRecord, StrategyRecord, StrategyStats, StrategyTracker,
    StrategyTrackerStats,
};
pub use nervous::{
    BtspLearner, BtspMemory, CircadianController, CircadianPhase, Forecast, ForecastPoint,
    Forecaster, GlobalWorkspace, HdcComputer, Hypervector, PhaseSchedule, WorkspaceItem,
    WtaNetwork,
};
pub use sona::{
    AdaptationFeedback, FisherInformation, KeywordPatternBank, LoraDelta, Pattern, PatternBank,
    PatternBankStats, PatternEntry, Sona, SonaStats,
};
