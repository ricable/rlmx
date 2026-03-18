use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Scheduling strategy selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Strategy {
    /// Use the RLM (Reinforcement Learning Model) scheduler.
    Rlm,
    /// Use the TRM (Tree of Reasoning Models) scheduler with a specific model.
    Trm(String),
    /// Automatically select the best strategy based on query characteristics.
    Auto,
    /// Hybrid: use a triage model to classify, then dispatch.
    Hybrid {
        triage: String,
        threshold: f32,
    },
}

impl Default for Strategy {
    fn default() -> Self {
        Strategy::Auto
    }
}

/// Configuration for the kernel scheduler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub default_strategy: Strategy,
    pub max_recursion_depth: usize,
    pub query_timeout: Duration,
    pub max_concurrent_processes: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            default_strategy: Strategy::Auto,
            max_recursion_depth: 10,
            query_timeout: Duration::from_secs(30),
            max_concurrent_processes: 64,
        }
    }
}

/// The core scheduler that dispatches queries to the appropriate strategy.
pub struct Scheduler {
    pub config: SchedulerConfig,
}

impl Scheduler {
    pub fn new(config: SchedulerConfig) -> Self {
        Self { config }
    }

    /// Determine which strategy to use for a given query.
    /// Returns the resolved strategy (never Auto -- Auto is resolved here).
    pub fn resolve_strategy(&self, query: &str, hint: Option<&Strategy>) -> Strategy {
        let strategy = hint.unwrap_or(&self.config.default_strategy);

        match strategy {
            Strategy::Auto => self.auto_select(query),
            other => other.clone(),
        }
    }

    /// Simple heuristic for auto-selecting a strategy.
    fn auto_select(&self, query: &str) -> Strategy {
        let len = query.len();
        let has_code = query.contains("```") || query.contains("fn ") || query.contains("def ");
        let is_question = query.trim_end().ends_with('?');

        if has_code || len > 500 {
            // Complex queries benefit from TRM decomposition.
            Strategy::Trm("default".into())
        } else if is_question && len < 200 {
            // Simple questions can use the faster RLM path.
            Strategy::Rlm
        } else {
            Strategy::Hybrid {
                triage: "auto-triage".into(),
                threshold: 0.5,
            }
        }
    }

    /// Check whether the current recursion depth is within limits.
    pub fn check_recursion_depth(&self, depth: usize) -> bool {
        depth < self.config.max_recursion_depth
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new(SchedulerConfig::default())
    }
}
