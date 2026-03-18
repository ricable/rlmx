//! Root and Sub-Agent implementation for the RLM recursive hierarchy.
//!
//! Each agent in the RLM hierarchy can reason about its task, retrieve
//! context, delegate to sub-agents, and return results. Agents are
//! organized in a tree structure with the root agent at the top.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::context::{ContextSegment, ContextWindow};
use crate::grammar::{RlmAction, ACTION_GRAMMAR_PROMPT};
use crate::vllm::{ChatMessage, ResponseFormat, VllmClient, VllmError};

/// The role of an agent in the recursive hierarchy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentRole {
    /// The root agent that receives the initial query.
    Root,
    /// A sub-agent spawned by a parent to handle a specific subtask.
    SubAgent {
        parent_id: String,
        task_description: String,
    },
}

impl AgentRole {
    /// Returns a human-readable description of the role.
    pub fn description(&self) -> String {
        match self {
            AgentRole::Root => "root agent".to_string(),
            AgentRole::SubAgent {
                task_description, ..
            } => format!("sub-agent: {}", task_description),
        }
    }

    /// Returns true if this is the root agent.
    pub fn is_root(&self) -> bool {
        matches!(self, AgentRole::Root)
    }
}

/// The result of an agent's execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    /// The agent's ID.
    pub agent_id: String,
    /// The final answer produced by this agent.
    pub answer: String,
    /// Chain of evidence supporting the answer.
    pub evidence_chain: Vec<String>,
    /// All actions taken during execution.
    pub actions_taken: Vec<String>,
    /// Results from any sub-agents that were spawned.
    pub sub_agent_results: Vec<AgentResult>,
    /// Total execution latency in milliseconds.
    pub latency_ms: u64,
    /// Confidence score of the answer (0.0 - 1.0).
    pub confidence: f64,
    /// Timestamp when the agent completed execution.
    pub completed_at: DateTime<Utc>,
}

/// An agent in the RLM recursive hierarchy.
///
/// Each agent has a role (root or sub-agent), a depth in the recursion tree,
/// and a set of capabilities that determine what actions it can take.
#[derive(Debug)]
pub struct RlmAgent {
    /// Unique identifier for this agent.
    pub id: String,
    /// The agent's role in the hierarchy.
    pub role: AgentRole,
    /// Current depth in the recursion tree (0 = root).
    pub depth: usize,
    /// Maximum recursion depth allowed.
    pub max_depth: usize,
    /// Maximum total actions (across all types) per agent run.
    pub max_actions: usize,
    /// Maximum number of Retrieve actions per agent run.
    pub max_retrievals: usize,
    /// Capabilities this agent is allowed to use.
    pub capabilities: Vec<String>,
    /// The context window for this agent.
    pub context_window: ContextWindow,
}

impl RlmAgent {
    /// Create a new RLM agent.
    pub fn new(role: AgentRole, depth: usize, capabilities: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role,
            depth,
            max_depth: 2,
            max_actions: 20,
            max_retrievals: 5,
            capabilities,
            context_window: ContextWindow::default(),
        }
    }

    /// Set the maximum recursion depth for this agent.
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = max_depth;
        self
    }

    /// Set the maximum total actions per agent run.
    pub fn with_max_actions(mut self, max_actions: usize) -> Self {
        self.max_actions = max_actions;
        self
    }

    /// Set the maximum number of Retrieve actions per agent run.
    pub fn with_max_retrievals(mut self, max_retrievals: usize) -> Self {
        self.max_retrievals = max_retrievals;
        self
    }

    /// Set a custom context window for this agent.
    pub fn with_context_window(mut self, context_window: ContextWindow) -> Self {
        self.context_window = context_window;
        self
    }

    /// Check if this agent can delegate to sub-agents (depth limit check).
    pub fn can_delegate(&self) -> bool {
        self.depth < self.max_depth
    }

    /// Check if this agent has a specific capability.
    pub fn has_capability(&self, cap: &str) -> bool {
        self.capabilities.contains(&cap.to_string())
    }

    /// Build the system prompt for this agent.
    fn build_system_prompt(&self) -> String {
        let role_desc = self.role.description();
        let caps = if self.capabilities.is_empty() {
            "all".to_string()
        } else {
            self.capabilities.join(", ")
        };

        let delegation_note = if self.can_delegate() {
            format!(
                "You may delegate subtasks to sub-agents (current depth: {}, max: {}).",
                self.depth, self.max_depth
            )
        } else {
            "You have reached maximum recursion depth. You must NOT delegate; resolve directly."
                .to_string()
        };

        format!(
            "You are an RLM agent ({role_desc}) in the RuVix recursive scheduling system.\n\
            Your capabilities: [{caps}]\n\
            {delegation_note}\n\n\
            {ACTION_GRAMMAR_PROMPT}"
        )
    }

    /// Run this agent on a query with the given vLLM client.
    ///
    /// The agent will:
    /// 1. Build a system prompt based on its role and capabilities
    /// 2. Include context segments in the prompt
    /// 3. Call the vLLM model for a structured action
    /// 4. Parse and execute the action
    /// 5. Recursively handle any delegated sub-tasks
    /// 6. Return the final result with evidence chain
    pub async fn run(
        &mut self,
        query: &str,
        context_segments: Vec<ContextSegment>,
        vllm_client: &VllmClient,
        temperature: Option<f64>,
    ) -> Result<AgentResult, AgentError> {
        let start = std::time::Instant::now();
        let mut actions_taken = Vec::new();
        let mut sub_agent_results = Vec::new();
        let mut evidence_chain = Vec::new();
        let mut action_count: usize = 0;
        let mut retrieval_count: usize = 0;

        // Add context segments to our window
        self.context_window
            .add_segments_by_priority(context_segments);

        let system_prompt = self.build_system_prompt();
        let messages = self.context_window.build_prompt(&system_prompt, query);

        // Call vLLM for initial action
        let response = vllm_client
            .chat_completion(
                messages,
                temperature,
                Some(2048),
                Some(ResponseFormat::json()),
            )
            .await
            .map_err(AgentError::VllmError)?;

        // Parse the action from the model's response
        let action = RlmAction::parse(&response).map_err(|e| {
            AgentError::ActionParseError(format!("failed to parse model output: {}", e))
        })?;

        actions_taken.push(action.to_prompt_string());

        // Execute the action
        let (answer, confidence) = self
            .execute_action(
                action,
                query,
                vllm_client,
                temperature,
                &mut actions_taken,
                &mut sub_agent_results,
                &mut evidence_chain,
                &mut action_count,
                &mut retrieval_count,
            )
            .await?;

        let latency_ms = start.elapsed().as_millis() as u64;

        Ok(AgentResult {
            agent_id: self.id.clone(),
            answer,
            evidence_chain,
            actions_taken,
            sub_agent_results,
            latency_ms,
            confidence,
            completed_at: Utc::now(),
        })
    }

    /// Execute a parsed RLM action.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    fn execute_action<'a>(
        &'a self,
        action: RlmAction,
        original_query: &'a str,
        vllm_client: &'a VllmClient,
        temperature: Option<f64>,
        actions_taken: &'a mut Vec<String>,
        sub_agent_results: &'a mut Vec<AgentResult>,
        evidence_chain: &'a mut Vec<String>,
        action_count: &'a mut usize,
        retrieval_count: &'a mut usize,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(String, f64), AgentError>> + Send + 'a>,
    > {
        Box::pin(async move {
            // Check global action limit for all action types
            *action_count += 1;
            if *action_count > self.max_actions {
                return Err(AgentError::ExecutionError(
                    "Maximum action depth exceeded".to_string(),
                ));
            }

            match action {
                RlmAction::Final {
                    answer,
                    evidence,
                    confidence,
                } => {
                    evidence_chain.extend(evidence);
                    Ok((answer, confidence))
                }

                RlmAction::Reason {
                    thought,
                    next_action,
                } => {
                    evidence_chain.push(format!("Reasoning: {}", thought));
                    actions_taken.push(next_action.to_prompt_string());
                    self.execute_action(
                        *next_action,
                        original_query,
                        vllm_client,
                        temperature,
                        actions_taken,
                        sub_agent_results,
                        evidence_chain,
                        action_count,
                        retrieval_count,
                    )
                    .await
                }

                RlmAction::Retrieve { query, k, .. } => {
                    // Check retrieval limit
                    *retrieval_count += 1;
                    if *retrieval_count > self.max_retrievals {
                        return Err(AgentError::ExecutionError(
                            "Maximum retrieval count exceeded".to_string(),
                        ));
                    }

                    // In a real system, this would call the kernel's vec_search syscall.
                    // For now, we record the retrieval attempt.
                    evidence_chain.push(format!("Retrieved {} results for: {}", k, query));

                    // After retrieval, we need another model call to process results
                    let follow_up_messages = vec![
                    ChatMessage::system(self.build_system_prompt()),
                    ChatMessage::user(format!(
                        "You previously searched for \"{}\" and got results. \
                        Based on the available context, provide your final answer.\n\nOriginal query: {}",
                        query, original_query
                    )),
                ];

                    let response = vllm_client
                        .chat_completion(
                            follow_up_messages,
                            temperature,
                            Some(2048),
                            Some(ResponseFormat::json()),
                        )
                        .await
                        .map_err(AgentError::VllmError)?;

                    let next_action = RlmAction::parse(&response).map_err(|e| {
                        AgentError::ActionParseError(format!("failed to parse follow-up: {}", e))
                    })?;
                    actions_taken.push(next_action.to_prompt_string());

                    self.execute_action(
                        next_action,
                        original_query,
                        vllm_client,
                        temperature,
                        actions_taken,
                        sub_agent_results,
                        evidence_chain,
                        action_count,
                        retrieval_count,
                    )
                    .await
                }

                RlmAction::Delegate {
                    task,
                    capabilities,
                    memory_scope: _,
                } => {
                    if !self.can_delegate() {
                        return Err(AgentError::MaxDepthExceeded(self.depth));
                    }

                    let sub_role = AgentRole::SubAgent {
                        parent_id: self.id.clone(),
                        task_description: task.clone(),
                    };
                    let sub_caps = if capabilities.is_empty() {
                        self.capabilities.clone()
                    } else {
                        capabilities
                    };

                    let mut sub_agent = RlmAgent::new(sub_role, self.depth + 1, sub_caps)
                        .with_max_depth(self.max_depth);

                    let sub_result = sub_agent
                        .run(&task, Vec::new(), vllm_client, temperature)
                        .await?;

                    evidence_chain.push(format!(
                        "Sub-agent {} returned: {}",
                        sub_result.agent_id, sub_result.answer
                    ));
                    let answer = sub_result.answer.clone();
                    let confidence = sub_result.confidence;
                    sub_agent_results.push(sub_result);

                    Ok((answer, confidence))
                }

                RlmAction::Commit {
                    action,
                    params,
                    confidence,
                } => {
                    // In a real system, this would call state_mutate on the kernel.
                    evidence_chain.push(format!(
                        "Committed action '{}' with params: {}",
                        action, params
                    ));
                    Ok((
                        format!("Executed mutation: {} (params: {})", action, params),
                        confidence,
                    ))
                }
            }
        }) // end Box::pin(async move)
    }
}

/// Errors that can occur during agent execution.
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("vLLM communication error: {0}")]
    VllmError(#[source] VllmError),

    #[error("failed to parse action from model output: {0}")]
    ActionParseError(String),

    #[error("maximum recursion depth exceeded at depth {0}")]
    MaxDepthExceeded(usize),

    #[error("agent execution failed: {0}")]
    ExecutionError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_role_creation() {
        let root = RlmAgent::new(
            AgentRole::Root,
            0,
            vec!["retrieve".into(), "delegate".into(), "commit".into()],
        );
        assert!(root.role.is_root());
        assert_eq!(root.depth, 0);
        assert!(root.can_delegate());
        assert!(root.has_capability("retrieve"));
        assert!(!root.has_capability("admin"));

        let sub = RlmAgent::new(
            AgentRole::SubAgent {
                parent_id: root.id.clone(),
                task_description: "analyze logs".into(),
            },
            2,
            vec!["retrieve".into()],
        )
        .with_max_depth(2);

        assert!(!sub.role.is_root());
        assert_eq!(sub.depth, 2);
        assert!(!sub.can_delegate()); // at max depth
        assert!(sub.role.description().contains("analyze logs"));
    }
}
