//! Constrained Action Grammar for RLM structured model output.
//!
//! Defines the action grammar that models must follow when generating
//! structured output. Actions are parsed from JSON and validated for
//! well-formedness before execution.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during grammar parsing and validation.
#[derive(Debug, Error)]
pub enum GrammarError {
    #[error("failed to parse action JSON: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("unknown action type: {0}")]
    UnknownAction(String),

    #[error("validation error: {0}")]
    ValidationError(String),

    #[error("missing required field: {0}")]
    MissingField(String),
}

/// The core action grammar for RLM agents.
///
/// Each action represents a step in the recursive decomposition process.
/// The model outputs one or more actions in JSON format, which are parsed
/// and executed by the RLM strategy engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RlmAction {
    /// Search context memory via vec_search syscall.
    Retrieve {
        query: String,
        #[serde(default = "default_k")]
        k: usize,
        #[serde(default)]
        filters: Option<serde_json::Value>,
    },

    /// Analyze retrieved context and plan next steps.
    Reason {
        thought: String,
        next_action: Box<RlmAction>,
    },

    /// Fork a sub-agent via process_fork with scoped capabilities.
    Delegate {
        task: String,
        #[serde(default)]
        capabilities: Vec<String>,
        #[serde(default = "default_memory_scope")]
        memory_scope: String,
    },

    /// Execute a state mutation via state_mutate.
    Commit {
        #[serde(rename = "action_name")]
        action: String,
        params: serde_json::Value,
        #[serde(default = "default_confidence")]
        confidence: f64,
    },

    /// Return final answer to the caller.
    Final {
        answer: String,
        #[serde(default)]
        evidence: Vec<String>,
        #[serde(default = "default_confidence")]
        confidence: f64,
    },
}

fn default_k() -> usize {
    5
}

fn default_memory_scope() -> String {
    "local".to_string()
}

fn default_confidence() -> f64 {
    0.5
}

impl RlmAction {
    /// Parse a JSON string into an RlmAction.
    pub fn parse(json_str: &str) -> Result<Self, GrammarError> {
        let action: RlmAction = serde_json::from_str(json_str)?;
        action.validate()?;
        Ok(action)
    }

    /// Parse a sequence of actions from a JSON array string.
    pub fn parse_sequence(json_str: &str) -> Result<Vec<Self>, GrammarError> {
        let actions: Vec<RlmAction> = serde_json::from_str(json_str)?;
        for action in &actions {
            action.validate()?;
        }
        Ok(actions)
    }

    /// Validate that an action is well-formed.
    pub fn validate(&self) -> Result<(), GrammarError> {
        match self {
            RlmAction::Retrieve { query, k, .. } => {
                if query.is_empty() {
                    return Err(GrammarError::ValidationError(
                        "retrieve query must not be empty".into(),
                    ));
                }
                if *k == 0 || *k > 100 {
                    return Err(GrammarError::ValidationError(
                        "k must be between 1 and 100".into(),
                    ));
                }
            }
            RlmAction::Reason {
                thought,
                next_action,
            } => {
                if thought.is_empty() {
                    return Err(GrammarError::ValidationError(
                        "reason thought must not be empty".into(),
                    ));
                }
                next_action.validate()?;
            }
            RlmAction::Delegate { task, .. } => {
                if task.is_empty() {
                    return Err(GrammarError::ValidationError(
                        "delegate task must not be empty".into(),
                    ));
                }
            }
            RlmAction::Commit {
                action, confidence, ..
            } => {
                if action.is_empty() {
                    return Err(GrammarError::ValidationError(
                        "commit action must not be empty".into(),
                    ));
                }
                if !(0.0..=1.0).contains(confidence) {
                    return Err(GrammarError::ValidationError(
                        "confidence must be between 0.0 and 1.0".into(),
                    ));
                }
            }
            RlmAction::Final {
                answer, confidence, ..
            } => {
                if answer.is_empty() {
                    return Err(GrammarError::ValidationError(
                        "final answer must not be empty".into(),
                    ));
                }
                if !(0.0..=1.0).contains(confidence) {
                    return Err(GrammarError::ValidationError(
                        "confidence must be between 0.0 and 1.0".into(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Convert the action to a prompt-friendly string representation.
    pub fn to_prompt_string(&self) -> String {
        match self {
            RlmAction::Retrieve { query, k, filters } => {
                let filter_str = filters
                    .as_ref()
                    .map(|f| format!(", filters: {}", f))
                    .unwrap_or_default();
                format!("RETRIEVE(query=\"{}\", k={}{})", query, k, filter_str)
            }
            RlmAction::Reason {
                thought,
                next_action,
            } => {
                format!(
                    "REASON(thought=\"{}\", then={})",
                    thought,
                    next_action.to_prompt_string()
                )
            }
            RlmAction::Delegate {
                task,
                capabilities,
                memory_scope,
            } => {
                format!(
                    "DELEGATE(task=\"{}\", caps=[{}], scope=\"{}\")",
                    task,
                    capabilities.join(", "),
                    memory_scope
                )
            }
            RlmAction::Commit {
                action,
                params,
                confidence,
            } => {
                format!(
                    "COMMIT(action=\"{}\", params={}, confidence={:.2})",
                    action, params, confidence
                )
            }
            RlmAction::Final {
                answer,
                evidence,
                confidence,
            } => {
                format!(
                    "FINAL(answer=\"{}\", evidence=[{}], confidence={:.2})",
                    answer,
                    evidence.join("; "),
                    confidence
                )
            }
        }
    }

    /// Returns the action type name as a string.
    pub fn action_type(&self) -> &'static str {
        match self {
            RlmAction::Retrieve { .. } => "retrieve",
            RlmAction::Reason { .. } => "reason",
            RlmAction::Delegate { .. } => "delegate",
            RlmAction::Commit { .. } => "commit",
            RlmAction::Final { .. } => "final",
        }
    }
}

/// The action grammar description for inclusion in system prompts.
pub const ACTION_GRAMMAR_PROMPT: &str = r#"You must respond with a JSON object containing exactly one action. Valid actions:

1. RETRIEVE - Search context memory
   {"action": "retrieve", "query": "<search query>", "k": <num_results>, "filters": <optional JSON>}

2. REASON - Analyze context and plan
   {"action": "reason", "thought": "<your reasoning>", "next_action": <another action object>}

3. DELEGATE - Fork a sub-agent for a subtask
   {"action": "delegate", "task": "<task description>", "capabilities": ["cap1", "cap2"], "memory_scope": "local|global"}

4. COMMIT - Execute a state mutation
   {"action": "commit", "action_name": "<mutation name>", "params": {<params>}, "confidence": <0.0-1.0>}

5. FINAL - Return your final answer
   {"action": "final", "answer": "<your answer>", "evidence": ["evidence1", "evidence2"], "confidence": <0.0-1.0>}

Always respond with valid JSON. Choose the most appropriate action for the current step."#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_retrieve_action() {
        let json = r#"{"action": "retrieve", "query": "user authentication flow", "k": 10}"#;
        let action = RlmAction::parse(json).expect("should parse retrieve");
        match action {
            RlmAction::Retrieve { query, k, filters } => {
                assert_eq!(query, "user authentication flow");
                assert_eq!(k, 10);
                assert!(filters.is_none());
            }
            _ => panic!("expected Retrieve action"),
        }
    }

    #[test]
    fn test_parse_delegate_action() {
        let json = r#"{
            "action": "delegate",
            "task": "analyze error logs",
            "capabilities": ["read_logs", "search"],
            "memory_scope": "local"
        }"#;
        let action = RlmAction::parse(json).expect("should parse delegate");
        match action {
            RlmAction::Delegate {
                task,
                capabilities,
                memory_scope,
            } => {
                assert_eq!(task, "analyze error logs");
                assert_eq!(capabilities, vec!["read_logs", "search"]);
                assert_eq!(memory_scope, "local");
            }
            _ => panic!("expected Delegate action"),
        }
    }

    #[test]
    fn test_parse_final_action() {
        let json = r#"{
            "action": "final",
            "answer": "The root cause is a null pointer dereference in module X.",
            "evidence": ["log line 42", "stack trace from crash dump"],
            "confidence": 0.92
        }"#;
        let action = RlmAction::parse(json).expect("should parse final");
        match action {
            RlmAction::Final {
                answer,
                evidence,
                confidence,
            } => {
                assert!(answer.contains("null pointer"));
                assert_eq!(evidence.len(), 2);
                assert!((confidence - 0.92).abs() < f64::EPSILON);
            }
            _ => panic!("expected Final action"),
        }
    }

    #[test]
    fn test_validation_rejects_empty_query() {
        let json = r#"{"action": "retrieve", "query": "", "k": 5}"#;
        let result = RlmAction::parse(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_rejects_invalid_confidence() {
        let json = r#"{"action": "final", "answer": "test", "evidence": [], "confidence": 1.5}"#;
        let result = RlmAction::parse(json);
        assert!(result.is_err());
    }
}
