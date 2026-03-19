//! DomainEvaluator trait for domain-specific evaluation of responses.

use std::collections::HashMap;

use crate::context::ContextSegment;

/// Trait for domain-specific evaluation of query responses.
pub trait DomainEvaluator: Send + Sync {
    /// Evaluate a response given the original query and context segments.
    fn evaluate(&self, query: &str, response: &str, context: &[ContextSegment])
        -> EvaluationResult;
}

/// Result of a domain evaluation.
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// Overall quality score (0.0 to 1.0).
    pub score: f64,
    /// Named metrics with their values.
    pub metrics: HashMap<String, f64>,
    /// Human-readable feedback string.
    pub feedback: String,
}

impl EvaluationResult {
    /// Create a new EvaluationResult.
    pub fn new(score: f64, feedback: impl Into<String>) -> Self {
        Self {
            score,
            metrics: HashMap::new(),
            feedback: feedback.into(),
        }
    }

    /// Add a metric (builder pattern).
    pub fn with_metric(mut self, name: impl Into<String>, value: f64) -> Self {
        self.metrics.insert(name.into(), value);
        self
    }
}
