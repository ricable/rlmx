//! All plugin type definitions for the RuVix plugin system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Strategy selection for handling a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Strategy {
    /// Use the Reasoning Language Model for complex analysis.
    Rlm,
    /// Use a specific Traditional ML model by name.
    Trm(String),
    /// Automatically select the best strategy.
    Auto,
    /// Hybrid approach with triage and threshold.
    Hybrid {
        /// Triage strategy name for initial classification.
        triage: String,
        /// Confidence threshold for escalation to RLM.
        threshold: f64,
    },
}

/// A preference mapping task patterns to strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPreference {
    /// Regex or keyword pattern to match incoming tasks.
    pub task_pattern: String,
    /// The strategy to use when the pattern matches.
    pub strategy: Strategy,
    /// Optional TRM model name to use (when strategy is Trm or Hybrid).
    pub trm_model: Option<String>,
    /// Rationale for why this strategy is preferred.
    pub rationale: String,
}

/// Parameter type for action definitions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParamType {
    /// String parameter.
    String,
    /// Floating-point parameter.
    Float,
    /// Integer parameter.
    Int,
    /// Boolean parameter.
    Bool,
    /// Array of strings.
    StringArray,
}

/// A parameter in an action definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionParam {
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    pub param_type: ParamType,
    /// Whether this parameter is required.
    pub required: bool,
    /// Description of the parameter.
    pub description: String,
}

/// Definition of an action that a plugin can execute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDefinition {
    /// Unique name of the action.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Parameters accepted by this action.
    pub params: Vec<ActionParam>,
}

impl ActionDefinition {
    /// Create a new ActionDefinition with a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            params: Vec::new(),
        }
    }

    /// Set the description (builder pattern).
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add a parameter (builder pattern).
    pub fn param(
        mut self,
        name: impl Into<String>,
        param_type: ParamType,
        required: bool,
        description: impl Into<String>,
    ) -> Self {
        self.params.push(ActionParam {
            name: name.into(),
            param_type,
            required,
            description: description.into(),
        });
        self
    }
}

/// Safety constraint applied to plugin actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SafetyConstraint {
    /// Enforce numeric bounds on a parameter.
    ParameterBound {
        /// Parameter name to constrain.
        param: String,
        /// Minimum allowed value (inclusive).
        min: f64,
        /// Maximum allowed value (inclusive).
        max: f64,
    },
    /// Guard against KPI degradation.
    KpiGuard {
        /// KPI metric name.
        metric: String,
        /// Maximum allowed degradation as a fraction (e.g., 0.05 = 5%).
        max_degradation: f64,
    },
    /// Require human approval for certain conditions.
    HumanEscalation {
        /// Condition description that triggers escalation.
        condition: String,
        /// Action names this escalation applies to. If empty, applies to no actions.
        actions: Vec<String>,
    },
    /// Rate limit on action execution.
    RateLimit {
        /// Action name to limit.
        action: String,
        /// Maximum number of executions in the window.
        max_count: u32,
        /// Time window in seconds.
        window_secs: u64,
    },
}

/// Configuration for embedding generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Name of the embedding model to use.
    pub model_name: String,
    /// Dimensionality of the embedding vectors.
    pub dimensions: usize,
}

/// Distance metric for vector similarity search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistanceMetric {
    /// Cosine similarity.
    Cosine,
    /// Euclidean (L2) distance.
    L2,
    /// Inner product distance.
    InnerProduct,
    /// Manhattan (L1) distance.
    Manhattan,
    /// Wasserstein (earth mover's) distance.
    Wasserstein,
    /// Poincaré hyperbolic distance.
    Poincare,
    /// Product manifold distance.
    ProductManifold,
}

/// Configuration for a Traditional ML (TRM) model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrmModelConfig {
    /// Model name identifier.
    pub name: String,
    /// Path to the model file or directory.
    pub path: String,
    /// Total number of trainable parameters.
    pub params: u64,
    /// Number of layers in the model.
    pub layers: u32,
    /// Input dimensionality.
    pub input_dim: u32,
    /// Number of output classes.
    pub output_classes: u32,
    /// Maximum reasoning cycles before halting.
    pub max_cycles: u32,
    /// Confidence threshold to halt early.
    pub halt_threshold: f64,
}

/// Definition of a segment type for the RVF (Rich Vector Format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentTypeDefinition {
    /// Name of the segment type.
    pub name: String,
    /// JSON schema describing the segment's metadata structure.
    pub schema: serde_json::Value,
}

/// Graph schema for knowledge graph integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSchema {
    /// Node types in the graph.
    pub node_types: Vec<NodeType>,
    /// Edge types in the graph.
    pub edge_types: Vec<EdgeType>,
}

/// A node type in the graph schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeType {
    /// Name of the node type.
    pub name: String,
    /// Properties associated with this node type.
    pub properties: HashMap<String, String>,
}

/// An edge type in the graph schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeType {
    /// Name of the edge type.
    pub name: String,
    /// Source node type name.
    pub source_type: String,
    /// Target node type name.
    pub target_type: String,
    /// Properties associated with this edge type.
    pub properties: HashMap<String, String>,
}
