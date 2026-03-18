//! RLM Strategy: Recursive Language Model scheduling policy for RuVix.
//!
//! The root agent receives a query + context metadata, reasons about what
//! information is needed, and uses a constrained action grammar to:
//! 1. RETRIEVE - Search context memory via vec_search syscall
//! 2. REASON - Analyze retrieved context and plan next steps
//! 3. DELEGATE - Fork sub-agent via process_fork with scoped capabilities
//! 4. COMMIT - Execute a state mutation via state_mutate
//! 5. FINAL - Return final answer to caller
//!
//! The strategy orchestrates the entire recursive decomposition process,
//! managing the root agent and its sub-agent tree.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::agent::{AgentError, AgentResult, AgentRole, RlmAgent};
use crate::context::{ContextSegment, ContextWindow};
use crate::vllm::{VllmClient, VllmConfig};

/// The result of an RLM strategy execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RlmResult {
    /// The final answer produced by the strategy.
    pub answer: String,
    /// Chain of evidence supporting the answer.
    pub evidence_chain: Vec<String>,
    /// Total number of agents spawned (including root).
    pub total_agents: usize,
    /// Maximum recursion depth reached during execution.
    pub max_depth_reached: usize,
    /// Total execution time in milliseconds.
    pub total_latency_ms: u64,
    /// Confidence score of the final answer (0.0 - 1.0).
    pub confidence: f64,
    /// The full result tree from the root agent.
    pub root_result: AgentResult,
    /// Timestamp when the strategy completed.
    pub completed_at: DateTime<Utc>,
}

/// Errors from the RLM strategy.
#[derive(Debug, thiserror::Error)]
pub enum StrategyError {
    #[error("agent execution failed: {0}")]
    AgentError(#[from] AgentError),

    #[error("vLLM client error: {0}")]
    VllmError(#[from] crate::vllm::VllmError),

    #[error("strategy configuration error: {0}")]
    ConfigError(String),
}

/// Configuration for the RLM strategy.
#[derive(Debug, Clone)]
pub struct RlmStrategyConfig {
    /// vLLM server endpoint URL.
    pub vllm_endpoint: String,
    /// Optional API key for the vLLM server.
    pub api_key: Option<String>,
    /// Model name to use for inference.
    pub model_name: String,
    /// Maximum recursion depth for sub-agent delegation.
    pub max_recursion_depth: usize,
    /// Maximum number of sub-agents that can be spawned.
    pub max_sub_agents: usize,
    /// Temperature for model sampling.
    pub temperature: f64,
    /// Maximum tokens for the context window.
    pub max_context_tokens: usize,
}

impl Default for RlmStrategyConfig {
    fn default() -> Self {
        Self {
            vllm_endpoint: "http://localhost:8000".to_string(),
            api_key: None,
            model_name: "default".to_string(),
            max_recursion_depth: 2,
            max_sub_agents: 5,
            temperature: 0.1,
            max_context_tokens: 8192,
        }
    }
}

/// RLM Strategy: the core recursive decomposition scheduling policy.
///
/// Orchestrates the recursive language model agent hierarchy. The root agent
/// receives the initial query, reasons about what information is needed,
/// and may delegate subtasks to child agents, each operating within a
/// scoped context window and capability set.
pub struct RlmStrategy {
    /// The vLLM client for model inference.
    vllm_client: VllmClient,
    /// Strategy configuration.
    config: RlmStrategyConfig,
}

impl RlmStrategy {
    /// Create a new RLM strategy with the given configuration.
    pub fn new(config: RlmStrategyConfig) -> Self {
        let vllm_config = VllmConfig {
            endpoint: config.vllm_endpoint.clone(),
            api_key: config.api_key.clone(),
            model: config.model_name.clone(),
            max_retries: 3,
            base_retry_delay_ms: 500,
        };

        Self {
            vllm_client: VllmClient::new(vllm_config),
            config,
        }
    }

    /// Create a strategy with default configuration pointing to a local vLLM server.
    pub fn local(model_name: impl Into<String>) -> Self {
        Self::new(RlmStrategyConfig {
            model_name: model_name.into(),
            ..Default::default()
        })
    }

    /// Get a reference to the strategy configuration.
    pub fn config(&self) -> &RlmStrategyConfig {
        &self.config
    }

    /// Get a reference to the vLLM client.
    pub fn vllm_client(&self) -> &VllmClient {
        &self.vllm_client
    }

    /// Execute the RLM strategy on a query with the given context segments.
    ///
    /// This is the main entry point for the recursive decomposition process:
    ///
    /// 1. Builds a system prompt with the action grammar and available context metadata
    /// 2. Calls vLLM via the OpenAI-compatible API
    /// 3. Parses the model's structured output into actions
    /// 4. Executes each action (RETRIEVE calls vec_search, DELEGATE calls process_fork, etc.)
    /// 5. Recursively handles sub-agent results
    /// 6. Returns the final answer with evidence chain
    pub async fn execute(
        &self,
        query: &str,
        context_segments: Vec<ContextSegment>,
    ) -> Result<RlmResult, StrategyError> {
        let start = std::time::Instant::now();

        info!(
            query = query,
            context_segments = context_segments.len(),
            max_depth = self.config.max_recursion_depth,
            "starting RLM strategy execution"
        );

        // Create the root agent with full capabilities
        let root_capabilities = vec![
            "retrieve".to_string(),
            "reason".to_string(),
            "delegate".to_string(),
            "commit".to_string(),
        ];

        let context_window = ContextWindow::new(self.config.max_context_tokens);

        let mut root_agent = RlmAgent::new(AgentRole::Root, 0, root_capabilities)
            .with_max_depth(self.config.max_recursion_depth)
            .with_context_window(context_window);

        debug!(
            agent_id = root_agent.id,
            "created root agent for RLM execution"
        );

        // Execute the root agent
        let root_result = root_agent
            .run(
                query,
                context_segments,
                &self.vllm_client,
                Some(self.config.temperature),
            )
            .await?;

        let total_latency_ms = start.elapsed().as_millis() as u64;
        let total_agents = count_agents(&root_result);
        let max_depth_reached = max_depth(&root_result, 0);

        info!(
            total_agents,
            max_depth_reached,
            total_latency_ms,
            confidence = root_result.confidence,
            "RLM strategy execution completed"
        );

        Ok(RlmResult {
            answer: root_result.answer.clone(),
            evidence_chain: root_result.evidence_chain.clone(),
            total_agents,
            max_depth_reached,
            total_latency_ms,
            confidence: root_result.confidence,
            root_result,
            completed_at: Utc::now(),
        })
    }

    /// Execute the strategy with a health check before starting.
    pub async fn execute_with_health_check(
        &self,
        query: &str,
        context_segments: Vec<ContextSegment>,
    ) -> Result<RlmResult, StrategyError> {
        self.vllm_client.health_check().await?;
        self.execute(query, context_segments).await
    }
}

/// Count the total number of agents in a result tree.
fn count_agents(result: &AgentResult) -> usize {
    1 + result
        .sub_agent_results
        .iter()
        .map(count_agents)
        .sum::<usize>()
}

/// Find the maximum depth reached in a result tree.
fn max_depth(result: &AgentResult, current_depth: usize) -> usize {
    if result.sub_agent_results.is_empty() {
        current_depth
    } else {
        result
            .sub_agent_results
            .iter()
            .map(|r| max_depth(r, current_depth + 1))
            .max()
            .unwrap_or(current_depth)
    }
}

/// A handle to the kernel for executing syscalls (placeholder for integration).
///
/// In the full RuVix system, this would provide access to:
/// - `vec_search` for context memory retrieval
/// - `process_fork` for spawning sub-agents
/// - `state_mutate` for state mutations
#[derive(Debug, Clone)]
pub struct KernelHandle {
    /// Identifier for the kernel instance.
    pub kernel_id: String,
}

impl KernelHandle {
    /// Create a new kernel handle.
    pub fn new(kernel_id: impl Into<String>) -> Self {
        Self {
            kernel_id: kernel_id.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_construction() {
        let strategy = RlmStrategy::local("llama-3-70b");
        assert_eq!(strategy.config().model_name, "llama-3-70b");
        assert_eq!(strategy.config().max_recursion_depth, 2);
        assert_eq!(strategy.config().max_sub_agents, 5);
        assert_eq!(strategy.vllm_client().endpoint(), "http://localhost:8000");
    }

    #[test]
    fn test_strategy_custom_config() {
        let config = RlmStrategyConfig {
            vllm_endpoint: "http://gpu-server:8080".to_string(),
            api_key: Some("sk-test-key".to_string()),
            model_name: "mixtral-8x7b".to_string(),
            max_recursion_depth: 3,
            max_sub_agents: 10,
            temperature: 0.2,
            max_context_tokens: 16384,
        };
        let strategy = RlmStrategy::new(config);
        assert_eq!(strategy.config().max_recursion_depth, 3);
        assert_eq!(strategy.config().max_sub_agents, 10);
        assert_eq!(strategy.config().max_context_tokens, 16384);
    }

    #[test]
    fn test_count_agents_and_depth() {
        let leaf = AgentResult {
            agent_id: "leaf".into(),
            answer: "leaf answer".into(),
            evidence_chain: vec![],
            actions_taken: vec![],
            sub_agent_results: vec![],
            latency_ms: 100,
            confidence: 0.8,
            completed_at: Utc::now(),
        };

        let root = AgentResult {
            agent_id: "root".into(),
            answer: "root answer".into(),
            evidence_chain: vec![],
            actions_taken: vec![],
            sub_agent_results: vec![leaf.clone(), leaf],
            latency_ms: 300,
            confidence: 0.9,
            completed_at: Utc::now(),
        };

        assert_eq!(count_agents(&root), 3);
        assert_eq!(max_depth(&root, 0), 1);
    }
}
