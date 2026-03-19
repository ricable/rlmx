//! rlmx-cognitive: Self-learning and cognitive layer for the RuVix kernel.
//!
//! This crate implements the cognitive self-improvement layer including:
//! - **SONA**: Self-Optimizing Neural Architecture (micro-LoRA, pattern bank)
//! - **DAG Optimizer**: Self-learning query execution strategy optimizer
//! - **Nervous System**: Bio-inspired components (BTSP, HDC, WTA, Circadian, Global Workspace)
//! - **Attention Selector**: Auto-selection among 39 attention mechanisms

pub mod attention;
pub mod cognitive_framework;
pub mod dag;
pub mod gnn;
pub mod nervous;
pub mod solver;
pub mod sona;
pub mod voice_patterns;

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
pub use voice_patterns::{
    AnonymizedPattern, EmotionBucket, EngagementEvent, EngagementTracker, FederatedAnonymizer,
    Modality, NotificationFatigueModel, VoiceEnrichedPattern, VoicePatternBank,
};

// Phase 3: Solver, GNN, Cognitive Framework (stub or real based on feature)
pub use cognitive_framework::{CognitiveFramework, CognitiveStats, StalePattern};
pub use gnn::{GraphEdge, GraphNode, LifeGraph, LifeGraphGnn, PatternPrediction};
pub use solver::{BudgetAllocation, BudgetConstraint, BudgetOptimizer, OptimizationResult};
