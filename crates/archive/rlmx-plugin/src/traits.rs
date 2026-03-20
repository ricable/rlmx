//! The DomainPlugin trait - core extension point for the RuVix plugin system.
//! Defined as specified in PRD Section 4.1.

use async_trait::async_trait;

use crate::adapter::IngestAdapter;
use crate::evaluator::DomainEvaluator;
use crate::types::{
    ActionDefinition, DistanceMetric, EmbeddingConfig, GraphSchema, SafetyConstraint,
    SegmentTypeDefinition, StrategyPreference, TrmModelConfig,
};

/// Core trait that all domain plugins must implement.
///
/// This trait defines the complete interface for extending the RuVix kernel
/// with domain-specific capabilities including data ingestion, strategy
/// selection, action execution, and safety constraints.
#[async_trait]
pub trait DomainPlugin: Send + Sync + 'static {
    /// Returns the unique name of this plugin.
    fn name(&self) -> &str;

    /// Returns the semantic version of this plugin.
    fn version(&self) -> semver::Version;

    /// Returns a human-readable description of this plugin.
    fn description(&self) -> &str;

    /// Returns the ingest adapters provided by this plugin.
    fn ingest_adapters(&self) -> Vec<Box<dyn IngestAdapter>>;

    /// Returns strategy preferences mapping task patterns to strategies.
    fn strategy_preferences(&self) -> Vec<StrategyPreference>;

    /// Returns action definitions that this plugin supports.
    fn action_extensions(&self) -> Vec<ActionDefinition>;

    /// Returns optional embedding configuration for this domain.
    fn embedding_config(&self) -> Option<EmbeddingConfig> {
        None
    }

    /// Returns the preferred distance metric for vector similarity.
    fn distance_metric(&self) -> Option<DistanceMetric> {
        None
    }

    /// Returns TRM model configurations used by this plugin.
    fn trm_models(&self) -> Vec<TrmModelConfig> {
        vec![]
    }

    /// Returns a system prompt extension with domain-specific instructions.
    fn system_prompt_extension(&self) -> String;

    /// Returns safety constraints that must be enforced for this plugin.
    fn safety_constraints(&self) -> Vec<SafetyConstraint>;

    /// Returns an optional domain-specific evaluator.
    fn evaluator(&self) -> Option<Box<dyn DomainEvaluator>> {
        None
    }

    /// Returns RVF segment type definitions for this domain.
    fn rvf_segment_types(&self) -> Vec<SegmentTypeDefinition> {
        vec![]
    }

    /// Returns an optional path to a knowledge base directory.
    fn knowledge_base(&self) -> Option<std::path::PathBuf> {
        None
    }

    /// Returns an optional graph schema for knowledge graph integration.
    fn graph_schema(&self) -> Option<GraphSchema> {
        None
    }
}
