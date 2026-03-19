use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The 14 specialized agent types in the RLMX swarm.
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
    /// Coordinates voice sessions: transcription, synthesis, intent routing.
    VoiceCoordinator,
    /// Manages marketplace operations: agent installation, updates, discovery.
    MarketplaceManager,
    /// Coordinates personal mesh network synchronization (from rlmx-mesh).
    MeshCoordinator,
    /// Manages federated learning contributions (from rlmx-federation).
    FederationAgent,
    /// Handles subscription billing and revenue operations (from rlmx-billing).
    BillingManager,
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
            AgentType::VoiceCoordinator,
            AgentType::MarketplaceManager,
            AgentType::MeshCoordinator,
            AgentType::FederationAgent,
            AgentType::BillingManager,
        ]
    }

    /// Whether this agent type can fork child processes (has ProcessFork).
    /// Per ADR-005, only Coordinator and Researcher have ProcessFork.
    pub fn can_fork(&self) -> bool {
        matches!(
            self,
            AgentType::Coordinator
                | AgentType::Researcher
                | AgentType::Experimenter
                | AgentType::VoiceCoordinator
                | AgentType::MeshCoordinator
        )
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
                | AgentType::MarketplaceManager
                | AgentType::BillingManager
        )
    }

    /// Preferred model tier for this agent.
    pub fn model_tier(&self) -> ModelTier {
        match self {
            AgentType::Coordinator
            | AgentType::Experimenter
            | AgentType::Analyst
            | AgentType::Trainer
            | AgentType::VoiceCoordinator => ModelTier::Medium,
            AgentType::Researcher | AgentType::Reviewer => ModelTier::ClaudeCode,
            AgentType::MarketplaceManager
            | AgentType::MeshCoordinator
            | AgentType::FederationAgent => ModelTier::Medium,
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
            | AgentType::Embedder
            | AgentType::VoiceCoordinator => "A",
            AgentType::Router | AgentType::Replicator => "B",
            AgentType::Monitor | AgentType::Validator => "C",
            AgentType::MeshCoordinator => "A",
            AgentType::FederationAgent => "B",
            AgentType::Worker
            | AgentType::Experimenter
            | AgentType::MarketplaceManager
            | AgentType::BillingManager => "Multi",
        }
    }

    /// A2A skill identifier (kebab-case) for this agent type.
    pub fn a2a_id(&self) -> &'static str {
        match self {
            AgentType::Coordinator => "coordinator",
            AgentType::Researcher => "researcher",
            AgentType::Router => "router",
            AgentType::Experimenter => "experimenter",
            AgentType::Worker => "worker",
            AgentType::Monitor => "monitor",
            AgentType::Reviewer => "reviewer",
            AgentType::Trainer => "trainer",
            AgentType::Validator => "validator",
            AgentType::Replicator => "replicator",
            AgentType::Embedder => "embedder",
            AgentType::Analyst => "analyst",
            AgentType::VoiceCoordinator => "voice-coordinator",
            AgentType::MarketplaceManager => "marketplace-manager",
            AgentType::MeshCoordinator => "mesh-coordinator",
            AgentType::FederationAgent => "federation-agent",
            AgentType::BillingManager => "billing-manager",
        }
    }

    /// Human-readable name for A2A agent cards.
    pub fn a2a_name(&self) -> &'static str {
        match self {
            AgentType::Coordinator => "Coordinator",
            AgentType::Researcher => "Researcher",
            AgentType::Router => "Router",
            AgentType::Experimenter => "Experimenter",
            AgentType::Worker => "Worker",
            AgentType::Monitor => "Monitor",
            AgentType::Reviewer => "Reviewer",
            AgentType::Trainer => "Trainer",
            AgentType::Validator => "Validator",
            AgentType::Replicator => "Replicator",
            AgentType::Embedder => "Embedder",
            AgentType::Analyst => "Analyst",
            AgentType::VoiceCoordinator => "Voice Coordinator",
            AgentType::MarketplaceManager => "Marketplace Manager",
            AgentType::MeshCoordinator => "Mesh Coordinator",
            AgentType::FederationAgent => "Federation Agent",
            AgentType::BillingManager => "Billing Manager",
        }
    }

    /// A2A skill description for this agent type.
    pub fn a2a_description(&self) -> &'static str {
        match self {
            AgentType::Coordinator => "Orchestrates multi-agent workflows, assigns tasks, and manages agent lifecycle",
            AgentType::Researcher => "Performs deep research, gathers information, and synthesizes findings",
            AgentType::Router => "Routes queries to appropriate agents based on intent and domain classification",
            AgentType::Experimenter => "Designs and runs experiments, A/B tests, and hypothesis validation",
            AgentType::Worker => "Executes general-purpose tasks and processes work items from the queue",
            AgentType::Monitor => "Monitors system health, resource usage, and performance metrics",
            AgentType::Reviewer => "Reviews agent outputs, validates quality, and provides feedback",
            AgentType::Trainer => "Trains and fine-tunes models, manages training data and pipelines",
            AgentType::Validator => "Validates data integrity, schema compliance, and proof verification",
            AgentType::Replicator => "Replicates data and state across nodes for redundancy and availability",
            AgentType::Embedder => "Generates vector embeddings for text, images, and structured data",
            AgentType::Analyst => "Analyzes data patterns, generates reports, and extracts insights",
            AgentType::VoiceCoordinator => "Manages voice sessions, transcription pipelines, and speech synthesis",
            AgentType::MarketplaceManager => "Manages marketplace listings, reviews, and agent distribution",
            AgentType::MeshCoordinator => "Coordinates personal device mesh topology and cross-device sync",
            AgentType::FederationAgent => "Participates in federated learning cycles with differential privacy",
            AgentType::BillingManager => "Manages subscriptions, usage tracking, and billing enforcement",
        }
    }

    /// A2A skill tags for this agent type.
    pub fn a2a_tags(&self) -> &'static [&'static str] {
        match self {
            AgentType::Coordinator => &["orchestration", "workflow", "management"],
            AgentType::Researcher => &["research", "analysis", "information-gathering"],
            AgentType::Router => &["routing", "classification", "intent"],
            AgentType::Experimenter => &["experimentation", "testing", "hypothesis"],
            AgentType::Worker => &["execution", "task-processing", "general"],
            AgentType::Monitor => &["monitoring", "health", "metrics"],
            AgentType::Reviewer => &["review", "quality", "validation"],
            AgentType::Trainer => &["training", "fine-tuning", "ml"],
            AgentType::Validator => &["validation", "verification", "compliance"],
            AgentType::Replicator => &["replication", "redundancy", "sync"],
            AgentType::Embedder => &["embeddings", "vectors", "semantic"],
            AgentType::Analyst => &["analysis", "reporting", "insights"],
            AgentType::VoiceCoordinator => &["voice", "speech", "transcription"],
            AgentType::MarketplaceManager => &["marketplace", "distribution", "publishing"],
            AgentType::MeshCoordinator => &["mesh", "devices", "sync"],
            AgentType::FederationAgent => &["federation", "privacy", "distributed-learning"],
            AgentType::BillingManager => &["billing", "subscriptions", "usage"],
        }
    }

    /// Maximum concurrent instances allowed.
    pub fn max_instances(&self) -> usize {
        match self {
            AgentType::Coordinator => 1,
            AgentType::Trainer => 1,
            AgentType::Reviewer | AgentType::Analyst => 2,
            AgentType::VoiceCoordinator => 4,
            AgentType::Researcher | AgentType::FederationAgent => 3,
            AgentType::MarketplaceManager
            | AgentType::MeshCoordinator
            | AgentType::BillingManager => 2,
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
            "voicecoordinator" | "voice_coordinator" => Ok(AgentType::VoiceCoordinator),
            "marketplacemanager" | "marketplace_manager" => Ok(AgentType::MarketplaceManager),
            "meshcoordinator" | "mesh_coordinator" => Ok(AgentType::MeshCoordinator),
            "federationagent" | "federation_agent" => Ok(AgentType::FederationAgent),
            "billingmanager" | "billing_manager" => Ok(AgentType::BillingManager),
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

/// Represents an installed marketplace agent with its metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInstallation {
    /// Unique installation identifier.
    pub installation_id: Uuid,
    /// The marketplace package name (e.g. "finance-advisor", "health-tracker").
    pub package_name: String,
    /// Semantic version of the installed agent package.
    pub version: String,
    /// The underlying agent type this installation runs as.
    pub agent_type: AgentType,
    /// Whether the agent is currently enabled.
    pub enabled: bool,
    /// When this agent was installed.
    pub installed_at: DateTime<Utc>,
    /// Optional configuration overrides from the marketplace listing.
    pub config_overrides: Option<serde_json::Value>,
}

/// Build A2A skills for all agent types, using [`AgentType`] as the single source of truth.
///
/// Returns a `Vec<rlmx_kernel::a2a::A2ASkill>` suitable for passing to
/// [`rlmx_kernel::a2a::AgentCardBuilder::with_skills`].
pub fn build_a2a_skills() -> Vec<rlmx_kernel::a2a::A2ASkill> {
    AgentType::all()
        .iter()
        .map(|at| rlmx_kernel::a2a::A2ASkill {
            id: at.a2a_id().to_string(),
            name: at.a2a_name().to_string(),
            description: at.a2a_description().to_string(),
            tags: at.a2a_tags().iter().map(|t| t.to_string()).collect(),
        })
        .collect()
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
    fn test_all_types_returns_17() {
        assert_eq!(AgentType::all().len(), 17);
    }

    #[test]
    fn test_can_fork() {
        // Per ADR-005, Coordinator, Researcher, and VoiceCoordinator have ProcessFork
        assert!(AgentType::Coordinator.can_fork());
        assert!(AgentType::Researcher.can_fork());
        assert!(AgentType::VoiceCoordinator.can_fork());
        assert!(!AgentType::Router.can_fork());
        assert!(AgentType::Experimenter.can_fork());
        assert!(!AgentType::Worker.can_fork());
        assert!(!AgentType::Monitor.can_fork());
        assert!(!AgentType::Reviewer.can_fork());
        assert!(!AgentType::Trainer.can_fork());
        assert!(!AgentType::Validator.can_fork());
        assert!(!AgentType::Replicator.can_fork());
        assert!(!AgentType::Embedder.can_fork());
        assert!(!AgentType::Analyst.can_fork());
        assert!(!AgentType::MarketplaceManager.can_fork());
        assert!(AgentType::MeshCoordinator.can_fork());
        assert!(!AgentType::FederationAgent.can_fork());
        assert!(!AgentType::BillingManager.can_fork());
    }

    #[test]
    fn test_can_mutate_state() {
        // Per ADR-005: Coordinator, Replicator, Experimenter, Trainer, MarketplaceManager
        assert!(AgentType::Coordinator.can_mutate_state());
        assert!(AgentType::Replicator.can_mutate_state());
        assert!(AgentType::Experimenter.can_mutate_state());
        assert!(AgentType::Trainer.can_mutate_state());
        assert!(AgentType::MarketplaceManager.can_mutate_state());
        assert!(!AgentType::Router.can_mutate_state());
        assert!(!AgentType::Worker.can_mutate_state());
        assert!(!AgentType::Monitor.can_mutate_state());
        assert!(!AgentType::Reviewer.can_mutate_state());
        assert!(!AgentType::Validator.can_mutate_state());
        assert!(!AgentType::Embedder.can_mutate_state());
        assert!(!AgentType::Analyst.can_mutate_state());
        assert!(!AgentType::Researcher.can_mutate_state());
        assert!(!AgentType::VoiceCoordinator.can_mutate_state());
        assert!(!AgentType::MeshCoordinator.can_mutate_state());
        assert!(!AgentType::FederationAgent.can_mutate_state());
        assert!(AgentType::BillingManager.can_mutate_state());
    }

    #[test]
    fn test_model_tier() {
        assert_eq!(AgentType::Coordinator.model_tier(), ModelTier::Medium);
        assert_eq!(AgentType::Researcher.model_tier(), ModelTier::ClaudeCode);
        assert_eq!(AgentType::Reviewer.model_tier(), ModelTier::ClaudeCode);
        assert_eq!(AgentType::Worker.model_tier(), ModelTier::Small);
        assert_eq!(AgentType::Router.model_tier(), ModelTier::Small);
        assert_eq!(AgentType::MeshCoordinator.model_tier(), ModelTier::Medium);
        assert_eq!(AgentType::FederationAgent.model_tier(), ModelTier::Medium);
        assert_eq!(AgentType::BillingManager.model_tier(), ModelTier::Small);
    }

    #[test]
    fn test_preferred_zone() {
        assert_eq!(AgentType::Coordinator.preferred_zone(), "A");
        assert_eq!(AgentType::Router.preferred_zone(), "B");
        assert_eq!(AgentType::Monitor.preferred_zone(), "C");
        assert_eq!(AgentType::Worker.preferred_zone(), "Multi");
        assert_eq!(AgentType::Experimenter.preferred_zone(), "Multi");
        assert_eq!(AgentType::MeshCoordinator.preferred_zone(), "A");
        assert_eq!(AgentType::FederationAgent.preferred_zone(), "B");
        assert_eq!(AgentType::BillingManager.preferred_zone(), "Multi");
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
        assert_eq!(AgentType::MeshCoordinator.max_instances(), 2);
        assert_eq!(AgentType::FederationAgent.max_instances(), 3);
        assert_eq!(AgentType::BillingManager.max_instances(), 2);
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

    #[test]
    fn test_build_a2a_skills_returns_17() {
        let skills = build_a2a_skills();
        assert_eq!(skills.len(), 17);
    }

    #[test]
    fn test_a2a_skill_ids_are_unique() {
        let skills = build_a2a_skills();
        let mut ids: Vec<&str> = skills.iter().map(|s| s.id.as_str()).collect();
        let len_before = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), len_before, "A2A skill IDs must be unique");
    }

    #[test]
    fn test_a2a_skills_have_nonempty_fields() {
        let skills = build_a2a_skills();
        for skill in &skills {
            assert!(!skill.id.is_empty(), "skill has empty id");
            assert!(!skill.name.is_empty(), "skill {} has empty name", skill.id);
            assert!(
                !skill.description.is_empty(),
                "skill {} has empty description",
                skill.id
            );
            assert!(!skill.tags.is_empty(), "skill {} has no tags", skill.id);
        }
    }

    #[test]
    fn test_a2a_methods_cover_all_variants() {
        // Ensures every AgentType variant has a2a_id/description/tags.
        // If a new variant is added to AgentType without updating these methods,
        // this test will fail to compile (non-exhaustive match).
        for at in AgentType::all() {
            assert!(!at.a2a_id().is_empty());
            assert!(!at.a2a_description().is_empty());
            assert!(!at.a2a_tags().is_empty());
        }
    }

    #[test]
    fn test_build_a2a_skills_integrates_with_agent_card_builder() {
        let skills = build_a2a_skills();
        let builder = rlmx_kernel::a2a::AgentCardBuilder::new("http://localhost:3000", "0.1.0")
            .with_skills(skills);
        let card = builder.build();
        assert_eq!(card.skills.len(), 17);
        assert!(card.description.contains("17"));
    }
}
