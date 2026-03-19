//! Core types for dynamic function evolution.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Unique identifier for an evolved function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FunctionId(pub Uuid);

impl FunctionId {
    /// Create a new random FunctionId.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for FunctionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for FunctionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Lifecycle status of an evolved function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FunctionStatus {
    Draft,
    Staging,
    Production,
    Deprecated,
    Killed,
}

impl FunctionStatus {
    /// Check whether a transition from `self` to `target` is valid.
    pub fn can_transition_to(self, target: FunctionStatus) -> bool {
        matches!(
            (self, target),
            (FunctionStatus::Draft, FunctionStatus::Staging)
                | (FunctionStatus::Staging, FunctionStatus::Production)
                | (FunctionStatus::Production, FunctionStatus::Deprecated)
                | (_, FunctionStatus::Killed)
        )
    }
}

impl fmt::Display for FunctionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            FunctionStatus::Draft => "draft",
            FunctionStatus::Staging => "staging",
            FunctionStatus::Production => "production",
            FunctionStatus::Deprecated => "deprecated",
            FunctionStatus::Killed => "killed",
        };
        write!(f, "{s}")
    }
}

/// Auto-scoring mode for a function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AutoScoreMode {
    /// Score every invocation.
    Auto,
    /// Score ~10% of invocations.
    Sampled,
    /// Score only when explicitly requested.
    Manual,
    /// No scoring.
    Off,
}

/// A versioned, evolvable function managed by the evolution engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolvedFunction {
    pub id: FunctionId,
    pub name: String,
    pub version: u32,
    pub code: String,
    pub goal: String,
    pub status: FunctionStatus,
    pub score_mode: AutoScoreMode,
    pub created_at: DateTime<Utc>,
    pub parent_id: Option<FunctionId>,
    pub metadata: HashMap<String, String>,
}

/// Result of scoring a single function execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreResult {
    pub overall: f64,
    pub correctness: f64,
    pub safety: f64,
    pub latency_score: f64,
    pub cost_score: f64,
}

/// Decision produced by the feedback engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FeedbackDecision {
    Keep,
    Improve { reason: String },
    Kill { reason: String },
}

/// Errors that can occur during function evolution.
#[derive(Debug, thiserror::Error)]
pub enum EvolveError {
    #[error("function not found")]
    FunctionNotFound,

    #[error("invalid transition from {from} to {to}")]
    InvalidTransition {
        from: FunctionStatus,
        to: FunctionStatus,
    },

    #[error("dual-gate failed at gate '{gate}': {reason}")]
    DualGateFailed { gate: String, reason: String },

    #[error("maximum improvement iterations reached")]
    MaxIterationsReached,

    #[error("DAG error: {0}")]
    DagError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn function_id_new_is_unique() {
        let a = FunctionId::new();
        let b = FunctionId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn function_id_default_works() {
        let id = FunctionId::default();
        assert!(!id.0.is_nil());
    }

    #[test]
    fn function_id_display() {
        let id = FunctionId(Uuid::nil());
        assert_eq!(id.to_string(), "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn function_id_serialize_roundtrip() {
        let id = FunctionId::new();
        let json = serde_json::to_string(&id).unwrap();
        let back: FunctionId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn status_valid_transitions() {
        assert!(FunctionStatus::Draft.can_transition_to(FunctionStatus::Staging));
        assert!(FunctionStatus::Staging.can_transition_to(FunctionStatus::Production));
        assert!(FunctionStatus::Production.can_transition_to(FunctionStatus::Deprecated));
    }

    #[test]
    fn status_any_to_killed() {
        for status in [
            FunctionStatus::Draft,
            FunctionStatus::Staging,
            FunctionStatus::Production,
            FunctionStatus::Deprecated,
        ] {
            assert!(status.can_transition_to(FunctionStatus::Killed));
        }
    }

    #[test]
    fn status_invalid_skip_draft_to_production() {
        assert!(!FunctionStatus::Draft.can_transition_to(FunctionStatus::Production));
    }

    #[test]
    fn status_invalid_backward() {
        assert!(!FunctionStatus::Production.can_transition_to(FunctionStatus::Staging));
        assert!(!FunctionStatus::Staging.can_transition_to(FunctionStatus::Draft));
        assert!(!FunctionStatus::Deprecated.can_transition_to(FunctionStatus::Production));
    }

    #[test]
    fn status_killed_cannot_transition() {
        assert!(!FunctionStatus::Killed.can_transition_to(FunctionStatus::Draft));
        assert!(!FunctionStatus::Killed.can_transition_to(FunctionStatus::Staging));
        assert!(!FunctionStatus::Killed.can_transition_to(FunctionStatus::Production));
    }

    #[test]
    fn status_display() {
        assert_eq!(FunctionStatus::Draft.to_string(), "draft");
        assert_eq!(FunctionStatus::Production.to_string(), "production");
        assert_eq!(FunctionStatus::Killed.to_string(), "killed");
    }

    #[test]
    fn status_serialize_roundtrip() {
        let status = FunctionStatus::Staging;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"staging\"");
        let back: FunctionStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, status);
    }

    #[test]
    fn auto_score_mode_serialize() {
        let mode = AutoScoreMode::Sampled;
        let json = serde_json::to_string(&mode).unwrap();
        assert_eq!(json, "\"sampled\"");
    }

    #[test]
    fn score_result_serialize() {
        let sr = ScoreResult {
            overall: 0.8,
            correctness: 1.0,
            safety: 0.9,
            latency_score: 0.5,
            cost_score: 0.6,
        };
        let json = serde_json::to_string(&sr).unwrap();
        let back: ScoreResult = serde_json::from_str(&json).unwrap();
        assert!((back.overall - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn feedback_decision_keep() {
        let d = FeedbackDecision::Keep;
        assert_eq!(d, FeedbackDecision::Keep);
    }

    #[test]
    fn feedback_decision_improve() {
        let d = FeedbackDecision::Improve {
            reason: "low score".into(),
        };
        match &d {
            FeedbackDecision::Improve { reason } => assert_eq!(reason, "low score"),
            _ => panic!("expected Improve"),
        }
    }

    #[test]
    fn feedback_decision_kill() {
        let d = FeedbackDecision::Kill {
            reason: "repeated failures".into(),
        };
        match &d {
            FeedbackDecision::Kill { reason } => assert_eq!(reason, "repeated failures"),
            _ => panic!("expected Kill"),
        }
    }

    #[test]
    fn evolve_error_display() {
        let e = EvolveError::FunctionNotFound;
        assert_eq!(e.to_string(), "function not found");

        let e = EvolveError::InvalidTransition {
            from: FunctionStatus::Draft,
            to: FunctionStatus::Production,
        };
        assert!(e.to_string().contains("draft"));
        assert!(e.to_string().contains("production"));
    }

    #[test]
    fn evolved_function_construct() {
        let f = EvolvedFunction {
            id: FunctionId::new(),
            name: "test_fn".into(),
            version: 1,
            code: "return 42;".into(),
            goal: "return the answer".into(),
            status: FunctionStatus::Draft,
            score_mode: AutoScoreMode::Auto,
            created_at: Utc::now(),
            parent_id: None,
            metadata: HashMap::new(),
        };
        assert_eq!(f.version, 1);
        assert_eq!(f.status, FunctionStatus::Draft);
    }
}
