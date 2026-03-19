//! rlmx-rlm: RLM Scheduling Policy for the RuVix kernel.
//!
//! Implements the Recursive Language Model (RLM) scheduling policy, which
//! decomposes complex queries through a hierarchy of LLM agents. Each agent
//! uses a constrained action grammar to retrieve context, reason about tasks,
//! delegate to sub-agents, and commit state mutations.
//!
//! # Architecture
//!
//! - **Strategy** (`strategy`): The top-level orchestrator that manages the
//!   recursive decomposition process.
//! - **Agent** (`agent`): Individual agents (root and sub-agents) that
//!   execute within the recursive hierarchy.
//! - **Grammar** (`grammar`): The constrained action grammar that models
//!   must follow for structured output.
//! - **Context** (`context`): Context window management with priority-based
//!   segment selection and token budgeting.
//! - **vLLM Client** (`vllm`): OpenAI-compatible HTTP client for vLLM
//!   inference servers.

pub mod agent;
pub mod context;
pub mod grammar;
pub mod strategy;
pub mod vllm;

// Re-export primary types for convenience.
pub use agent::{AgentResult, AgentRole, RlmAgent};
pub use context::{ContextSegment, ContextWindow};
pub use grammar::RlmAction;
pub use strategy::{RlmResult, RlmStrategy, RlmStrategyConfig};
pub use vllm::{ChatMessage, VllmClient, VllmConfig};
