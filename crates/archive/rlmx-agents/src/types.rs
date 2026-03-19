use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The 12 specialized agent types in the RLMX swarm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    Coordinator,
    Researcher,
    Router,
    Experimenter,
    Worker,
    Monitor,
    Reviewer,
    Trainer,
    Validator,
    Replicator,
    Embedder,
    Analyst,
}

impl AgentType {
    pub fn all() -> &'static [AgentType] {
        &[
            AgentType::Coordinator,
            AgentType::Researcher,
            AgentType::Router,
            AgentType::Experimenter,
            AgentType::Worker,
            AgentType::Monitor,
            AgentType::Reviewer,
            AgentType::Trainer,
            AgentType::Validator,
            AgentType::Replicator,
            AgentType::Embedder,
            AgentType::Analyst,
        ]
    }

    /// Whether this agent type can fork child processes (has ProcessFork).
    /// Per ADR-005, only Coordinator and Researcher have ProcessFork.
    pub fn can_fork(&self) -> bool {
        matches!(self, AgentType::Coordinator | AgentType::Researcher)
    }

    /// Whether this agent type can mutate kernel state (has StateMutate).
    /// Per ADR-005: Coordinator, Replicator, Experimenter, and Trainer.
    pub fn can_mutate_state(&self) -> bool {
        matches!(
            self,
            AgentType::Coordinator
                | AgentType::Replicator
                | AgentType::Experimenter
                | AgentType::Trainer
        )
    }

    /// Preferred model tier for this agent.
    pub fn model_tier(&self) -> ModelTier {
        match self {
            AgentType::Coordinator
            | AgentType::Experimenter
            | AgentType::Analyst
            | AgentType::Trainer => ModelTier::Medium,
            AgentType::Researcher | AgentType::Reviewer => ModelTier::ClaudeCode,
            _ => ModelTier::Small,
        }
    }

    /// Preferred zone for this agent.
    pub fn preferred_zone(&self) -> &'static str {
        match self {
            AgentType::Coordinator
            | AgentType::Researcher
            | AgentType::Reviewer
            | AgentType::Trainer
            | AgentType::Analyst
            | AgentType::Embedder => "A",
            AgentType::Router | AgentType::Replicator => "B",
            AgentType::Monitor | AgentType::Validator => "C",
            AgentType::Worker | AgentType::Experimenter => "Multi",
        }
    }

    /// Maximum concurrent instances allowed.
    pub fn max_instances(&self) -> usize {
        match self {
            AgentType::Coordinator => 1,
            AgentType::Trainer => 1,
            AgentType::Reviewer | AgentType::Analyst => 2,
            AgentType::Researcher => 3,
            AgentType::Experimenter => 8,
            _ => 16,
        }
    }
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for AgentType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "coordinator" => Ok(AgentType::Coordinator),
            "researcher" => Ok(AgentType::Researcher),
            "router" => Ok(AgentType::Router),
            "experimenter" => Ok(AgentType::Experimenter),
            "worker" => Ok(AgentType::Worker),
            "monitor" => Ok(AgentType::Monitor),
            "reviewer" => Ok(AgentType::Reviewer),
            "trainer" => Ok(AgentType::Trainer),
            "validator" => Ok(AgentType::Validator),
            "replicator" => Ok(AgentType::Replicator),
            "embedder" => Ok(AgentType::Embedder),
            "analyst" => Ok(AgentType::Analyst),
            _ => Err(format!("Unknown agent type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelTier {
    Small,
    ClaudeCode,
    Medium,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentStatus {
    Pending,
    Starting,
    Running,
    Paused,
    Stopping,
    Terminated,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AgentId(pub Uuid);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_type: AgentType,
    pub name: Option<String>,
    pub task: Option<String>,
    pub zone: Option<String>,
    pub max_runtime_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: AgentId,
    pub agent_type: AgentType,
    pub name: String,
    pub status: AgentStatus,
    pub task: Option<String>,
    pub zone: String,
    pub spawned_at: DateTime<Utc>,
    pub parent_id: Option<AgentId>,
    pub children: Vec<AgentId>,
    pub model_tier: ModelTier,
    pub metrics: AgentMetrics,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentMetrics {
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub uptime_secs: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub from: AgentId,
    pub to: AgentId,
    pub content: MessageContent,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Task(String),
    Result(serde_json::Value),
    Query(String),
    Response(serde_json::Value),
    Heartbeat,
    Shutdown,
}

#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Agent not found: {0}")]
    NotFound(String),
    #[error("Max instances exceeded for {agent_type}: limit {max}")]
    MaxInstancesExceeded { agent_type: AgentType, max: usize },
    #[error("Agent type {child} cannot be spawned by {parent}")]
    SpawnNotAllowed { parent: AgentType, child: AgentType },
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Agent already terminated")]
    AlreadyTerminated,
    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_types_returns_12() {
        assert_eq!(AgentType::all().len(), 12);
    }

    #[test]
    fn test_can_fork() {
        // Per ADR-005, only Coordinator and Researcher have ProcessFork
        assert!(AgentType::Coordinator.can_fork());
        assert!(AgentType::Researcher.can_fork());
        assert!(!AgentType::Router.can_fork());
        assert!(!AgentType::Experimenter.can_fork());
        assert!(!AgentType::Worker.can_fork());
        assert!(!AgentType::Monitor.can_fork());
        assert!(!AgentType::Reviewer.can_fork());
        assert!(!AgentType::Trainer.can_fork());
        assert!(!AgentType::Validator.can_fork());
        assert!(!AgentType::Replicator.can_fork());
        assert!(!AgentType::Embedder.can_fork());
        assert!(!AgentType::Analyst.can_fork());
    }

    #[test]
    fn test_can_mutate_state() {
        // Per ADR-005: Coordinator, Replicator, Experimenter, Trainer
        assert!(AgentType::Coordinator.can_mutate_state());
        assert!(AgentType::Replicator.can_mutate_state());
        assert!(AgentType::Experimenter.can_mutate_state());
        assert!(AgentType::Trainer.can_mutate_state());
        assert!(!AgentType::Router.can_mutate_state());
        assert!(!AgentType::Worker.can_mutate_state());
        assert!(!AgentType::Monitor.can_mutate_state());
        assert!(!AgentType::Reviewer.can_mutate_state());
        assert!(!AgentType::Validator.can_mutate_state());
        assert!(!AgentType::Embedder.can_mutate_state());
        assert!(!AgentType::Analyst.can_mutate_state());
        assert!(!AgentType::Researcher.can_mutate_state());
    }

    #[test]
    fn test_model_tier() {
        assert_eq!(AgentType::Coordinator.model_tier(), ModelTier::Medium);
        assert_eq!(AgentType::Researcher.model_tier(), ModelTier::ClaudeCode);
        assert_eq!(AgentType::Reviewer.model_tier(), ModelTier::ClaudeCode);
        assert_eq!(AgentType::Worker.model_tier(), ModelTier::Small);
        assert_eq!(AgentType::Router.model_tier(), ModelTier::Small);
    }

    #[test]
    fn test_preferred_zone() {
        assert_eq!(AgentType::Coordinator.preferred_zone(), "A");
        assert_eq!(AgentType::Router.preferred_zone(), "B");
        assert_eq!(AgentType::Monitor.preferred_zone(), "C");
        assert_eq!(AgentType::Worker.preferred_zone(), "Multi");
        assert_eq!(AgentType::Experimenter.preferred_zone(), "Multi");
    }

    #[test]
    fn test_max_instances() {
        assert_eq!(AgentType::Coordinator.max_instances(), 1);
        assert_eq!(AgentType::Trainer.max_instances(), 1);
        assert_eq!(AgentType::Reviewer.max_instances(), 2);
        assert_eq!(AgentType::Analyst.max_instances(), 2);
        assert_eq!(AgentType::Researcher.max_instances(), 3);
        assert_eq!(AgentType::Experimenter.max_instances(), 8);
        assert_eq!(AgentType::Worker.max_instances(), 16);
    }

    #[test]
    fn test_from_str() {
        assert_eq!(
            "coordinator".parse::<AgentType>().unwrap(),
            AgentType::Coordinator
        );
        assert_eq!(
            "RESEARCHER".parse::<AgentType>().unwrap(),
            AgentType::Researcher
        );
        assert_eq!("Worker".parse::<AgentType>().unwrap(), AgentType::Worker);
        assert!("unknown".parse::<AgentType>().is_err());
    }

    #[test]
    fn test_display() {
        assert_eq!(AgentType::Coordinator.to_string(), "Coordinator");
        assert_eq!(AgentType::Worker.to_string(), "Worker");
    }

    #[test]
    fn test_agent_id_default() {
        let id1 = AgentId::default();
        let id2 = AgentId::default();
        assert_ne!(id1.0, id2.0);
    }
}
