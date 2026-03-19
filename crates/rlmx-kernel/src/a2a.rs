//! A2A (Agent-to-Agent) protocol types and agent card builder (ADR-034).
//!
//! Provides core types for the A2A protocol: task state machine, skills,
//! agent cards, and a builder that maps all 17 RLMX agent types to A2A skills.

use serde::{Deserialize, Serialize};

/// A2A task states following the protocol spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaskState {
    Submitted,
    Working,
    InputRequired,
    Completed,
    Cancelled,
    Failed,
}

impl TaskState {
    /// Returns `true` if this is a terminal state (no further transitions allowed).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Failed)
    }

    /// Check whether a transition from `self` to `next` is valid.
    pub fn can_transition_to(&self, next: TaskState) -> bool {
        match self {
            Self::Submitted => matches!(next, Self::Working | Self::Cancelled),
            Self::Working => matches!(
                next,
                Self::Completed | Self::Failed | Self::InputRequired | Self::Cancelled
            ),
            Self::InputRequired => matches!(next, Self::Working | Self::Cancelled),
            Self::Completed | Self::Cancelled | Self::Failed => false,
        }
    }
}

/// A skill advertised by an agent via A2A.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2ASkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

/// An A2A agent card describing an agent's capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCard {
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: String,
    pub capabilities: AgentCapabilities,
    pub skills: Vec<A2ASkill>,
    pub authentication: AuthenticationInfo,
}

/// Capabilities advertised in an agent card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapabilities {
    pub streaming: bool,
    pub push_notifications: bool,
    pub state_transition_history: bool,
}

/// Authentication information for an agent card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationInfo {
    pub schemes: Vec<String>,
}

/// Agent type descriptor used by [`AgentCardBuilder`] to generate skills.
struct AgentTypeDescriptor {
    id: &'static str,
    name: &'static str,
    description: &'static str,
    tags: &'static [&'static str],
}

/// All 17 RLMX agent types with their A2A skill metadata.
const AGENT_TYPE_DESCRIPTORS: &[AgentTypeDescriptor] = &[
    AgentTypeDescriptor {
        id: "coordinator",
        name: "Coordinator",
        description: "Orchestrates multi-agent workflows, assigns tasks, and manages agent lifecycle",
        tags: &["orchestration", "workflow", "management"],
    },
    AgentTypeDescriptor {
        id: "researcher",
        name: "Researcher",
        description: "Performs deep research, gathers information, and synthesizes findings",
        tags: &["research", "analysis", "information-gathering"],
    },
    AgentTypeDescriptor {
        id: "router",
        name: "Router",
        description: "Routes queries to appropriate agents based on intent and domain classification",
        tags: &["routing", "classification", "intent"],
    },
    AgentTypeDescriptor {
        id: "experimenter",
        name: "Experimenter",
        description: "Designs and runs experiments, A/B tests, and hypothesis validation",
        tags: &["experimentation", "testing", "hypothesis"],
    },
    AgentTypeDescriptor {
        id: "worker",
        name: "Worker",
        description: "Executes general-purpose tasks and processes work items from the queue",
        tags: &["execution", "task-processing", "general"],
    },
    AgentTypeDescriptor {
        id: "monitor",
        name: "Monitor",
        description: "Monitors system health, resource usage, and performance metrics",
        tags: &["monitoring", "health", "metrics"],
    },
    AgentTypeDescriptor {
        id: "reviewer",
        name: "Reviewer",
        description: "Reviews agent outputs, validates quality, and provides feedback",
        tags: &["review", "quality", "validation"],
    },
    AgentTypeDescriptor {
        id: "trainer",
        name: "Trainer",
        description: "Trains and fine-tunes models, manages training data and pipelines",
        tags: &["training", "fine-tuning", "ml"],
    },
    AgentTypeDescriptor {
        id: "validator",
        name: "Validator",
        description: "Validates data integrity, schema compliance, and proof verification",
        tags: &["validation", "verification", "compliance"],
    },
    AgentTypeDescriptor {
        id: "replicator",
        name: "Replicator",
        description: "Replicates data and state across nodes for redundancy and availability",
        tags: &["replication", "redundancy", "sync"],
    },
    AgentTypeDescriptor {
        id: "embedder",
        name: "Embedder",
        description: "Generates vector embeddings for text, images, and structured data",
        tags: &["embeddings", "vectors", "semantic"],
    },
    AgentTypeDescriptor {
        id: "analyst",
        name: "Analyst",
        description: "Analyzes data patterns, generates reports, and extracts insights",
        tags: &["analysis", "reporting", "insights"],
    },
    AgentTypeDescriptor {
        id: "voice-coordinator",
        name: "Voice Coordinator",
        description: "Manages voice sessions, transcription pipelines, and speech synthesis",
        tags: &["voice", "speech", "transcription"],
    },
    AgentTypeDescriptor {
        id: "marketplace-manager",
        name: "Marketplace Manager",
        description: "Manages marketplace listings, reviews, and agent distribution",
        tags: &["marketplace", "distribution", "publishing"],
    },
    AgentTypeDescriptor {
        id: "mesh-coordinator",
        name: "Mesh Coordinator",
        description: "Coordinates personal device mesh topology and cross-device sync",
        tags: &["mesh", "devices", "sync"],
    },
    AgentTypeDescriptor {
        id: "federation-agent",
        name: "Federation Agent",
        description: "Participates in federated learning cycles with differential privacy",
        tags: &["federation", "privacy", "distributed-learning"],
    },
    AgentTypeDescriptor {
        id: "billing-manager",
        name: "Billing Manager",
        description: "Manages subscriptions, usage tracking, and billing enforcement",
        tags: &["billing", "subscriptions", "usage"],
    },
];

/// Builder for [`AgentCard`] from kernel agent registry.
pub struct AgentCardBuilder {
    base_url: String,
    version: String,
}

impl AgentCardBuilder {
    /// Create a new builder with the given base URL and version.
    pub fn new(base_url: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            version: version.into(),
        }
    }

    /// Build an agent card exposing all 17 agent types as A2A skills.
    pub fn build(&self) -> AgentCard {
        let skills: Vec<A2ASkill> = AGENT_TYPE_DESCRIPTORS
            .iter()
            .map(|desc| A2ASkill {
                id: desc.id.to_string(),
                name: desc.name.to_string(),
                description: desc.description.to_string(),
                tags: desc.tags.iter().map(|t| t.to_string()).collect(),
            })
            .collect();

        AgentCard {
            name: "RLMX Agent".to_string(),
            version: self.version.clone(),
            description: "RLMX cognition kernel agent with 17 specialized agent types".to_string(),
            url: self.base_url.clone(),
            capabilities: AgentCapabilities {
                streaming: false,
                push_notifications: false,
                state_transition_history: true,
            },
            skills,
            authentication: AuthenticationInfo {
                schemes: vec!["bearer".to_string()],
            },
        }
    }

    /// Build an agent card with only the specified skill IDs.
    pub fn build_filtered(&self, skill_ids: &[&str]) -> AgentCard {
        let mut card = self.build();
        card.skills.retain(|s| skill_ids.contains(&s.id.as_str()));
        card
    }
}

/// Message part types for A2A protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Part {
    Text { text: String },
    File { name: String, mime_type: String, data: String },
    Data { data: serde_json::Value },
}

/// A2A protocol message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    pub role: String,
    pub parts: Vec<Part>,
}

/// A2A task with state machine lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2ATask {
    pub id: String,
    pub state: TaskState,
    pub messages: Vec<A2AMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<Part>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_state_terminal() {
        assert!(!TaskState::Submitted.is_terminal());
        assert!(!TaskState::Working.is_terminal());
        assert!(!TaskState::InputRequired.is_terminal());
        assert!(TaskState::Completed.is_terminal());
        assert!(TaskState::Cancelled.is_terminal());
        assert!(TaskState::Failed.is_terminal());
    }

    #[test]
    fn task_state_transitions_from_submitted() {
        assert!(TaskState::Submitted.can_transition_to(TaskState::Working));
        assert!(TaskState::Submitted.can_transition_to(TaskState::Cancelled));
        assert!(!TaskState::Submitted.can_transition_to(TaskState::Completed));
        assert!(!TaskState::Submitted.can_transition_to(TaskState::Failed));
        assert!(!TaskState::Submitted.can_transition_to(TaskState::InputRequired));
    }

    #[test]
    fn task_state_transitions_from_working() {
        assert!(TaskState::Working.can_transition_to(TaskState::Completed));
        assert!(TaskState::Working.can_transition_to(TaskState::Failed));
        assert!(TaskState::Working.can_transition_to(TaskState::InputRequired));
        assert!(TaskState::Working.can_transition_to(TaskState::Cancelled));
        assert!(!TaskState::Working.can_transition_to(TaskState::Submitted));
    }

    #[test]
    fn task_state_transitions_from_input_required() {
        assert!(TaskState::InputRequired.can_transition_to(TaskState::Working));
        assert!(TaskState::InputRequired.can_transition_to(TaskState::Cancelled));
        assert!(!TaskState::InputRequired.can_transition_to(TaskState::Completed));
        assert!(!TaskState::InputRequired.can_transition_to(TaskState::Submitted));
    }

    #[test]
    fn terminal_states_cannot_transition() {
        for terminal in [TaskState::Completed, TaskState::Cancelled, TaskState::Failed] {
            for target in [
                TaskState::Submitted,
                TaskState::Working,
                TaskState::InputRequired,
                TaskState::Completed,
                TaskState::Cancelled,
                TaskState::Failed,
            ] {
                assert!(
                    !terminal.can_transition_to(target),
                    "{:?} should not transition to {:?}",
                    terminal,
                    target
                );
            }
        }
    }

    #[test]
    fn agent_card_builder_creates_17_skills() {
        let builder = AgentCardBuilder::new("http://localhost:3000", "0.1.0");
        let card = builder.build();
        assert_eq!(card.skills.len(), 17);
        assert_eq!(card.version, "0.1.0");
        assert_eq!(card.url, "http://localhost:3000");
    }

    #[test]
    fn agent_card_skills_have_unique_ids() {
        let builder = AgentCardBuilder::new("http://localhost:3000", "0.1.0");
        let card = builder.build();
        let mut ids: Vec<&str> = card.skills.iter().map(|s| s.id.as_str()).collect();
        let len_before = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), len_before, "skill IDs must be unique");
    }

    #[test]
    fn agent_card_skills_have_nonempty_descriptions() {
        let builder = AgentCardBuilder::new("http://localhost:3000", "0.1.0");
        let card = builder.build();
        for skill in &card.skills {
            assert!(!skill.description.is_empty(), "skill {} has empty description", skill.id);
            assert!(!skill.name.is_empty(), "skill {} has empty name", skill.id);
            assert!(!skill.tags.is_empty(), "skill {} has no tags", skill.id);
        }
    }

    #[test]
    fn agent_card_has_bearer_auth() {
        let builder = AgentCardBuilder::new("http://localhost:3000", "0.1.0");
        let card = builder.build();
        assert!(card.authentication.schemes.contains(&"bearer".to_string()));
    }

    #[test]
    fn agent_card_capabilities_defaults() {
        let builder = AgentCardBuilder::new("http://localhost:3000", "0.1.0");
        let card = builder.build();
        assert!(!card.capabilities.streaming);
        assert!(!card.capabilities.push_notifications);
        assert!(card.capabilities.state_transition_history);
    }

    #[test]
    fn agent_card_builder_filtered() {
        let builder = AgentCardBuilder::new("http://localhost:3000", "0.1.0");
        let card = builder.build_filtered(&["coordinator", "worker"]);
        assert_eq!(card.skills.len(), 2);
        assert!(card.skills.iter().any(|s| s.id == "coordinator"));
        assert!(card.skills.iter().any(|s| s.id == "worker"));
    }

    #[test]
    fn agent_card_builder_filtered_empty() {
        let builder = AgentCardBuilder::new("http://localhost:3000", "0.1.0");
        let card = builder.build_filtered(&[]);
        assert!(card.skills.is_empty());
    }

    #[test]
    fn agent_card_serializes_to_json() {
        let builder = AgentCardBuilder::new("http://localhost:3000", "0.1.0");
        let card = builder.build();
        let json = serde_json::to_string(&card).expect("serialize");
        assert!(json.contains("RLMX Agent"));
        assert!(json.contains("coordinator"));
        assert!(json.contains("bearer"));
    }

    #[test]
    fn task_state_serializes_correctly() {
        let state = TaskState::InputRequired;
        let json = serde_json::to_string(&state).expect("serialize");
        assert_eq!(json, "\"input-required\"");
    }

    #[test]
    fn a2a_skill_roundtrip() {
        let skill = A2ASkill {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "A test skill".to_string(),
            tags: vec!["test".to_string()],
        };
        let json = serde_json::to_string(&skill).expect("serialize");
        let deserialized: A2ASkill = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.id, "test");
        assert_eq!(deserialized.tags.len(), 1);
    }

    #[test]
    fn agent_card_roundtrip() {
        let builder = AgentCardBuilder::new("http://localhost:3000", "0.1.0");
        let card = builder.build();
        let json = serde_json::to_string(&card).expect("serialize");
        let deserialized: AgentCard = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.name, card.name);
        assert_eq!(deserialized.skills.len(), 17);
    }

    #[test]
    fn task_state_deserializes() {
        let state: TaskState = serde_json::from_str("\"working\"").expect("deserialize");
        assert_eq!(state, TaskState::Working);
    }

    #[test]
    fn part_text_roundtrip() {
        let part = Part::Text { text: "hello".to_string() };
        let json = serde_json::to_string(&part).expect("serialize");
        assert!(json.contains("\"type\":\"text\""));
        let deserialized: Part = serde_json::from_str(&json).expect("deserialize");
        match deserialized {
            Part::Text { text } => assert_eq!(text, "hello"),
            _ => panic!("expected Part::Text"),
        }
    }

    #[test]
    fn part_file_and_data_roundtrip() {
        let file_part = Part::File {
            name: "report.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
            data: "base64data".to_string(),
        };
        let json = serde_json::to_string(&file_part).expect("serialize");
        let deserialized: Part = serde_json::from_str(&json).expect("deserialize");
        match deserialized {
            Part::File { name, mime_type, .. } => {
                assert_eq!(name, "report.pdf");
                assert_eq!(mime_type, "application/pdf");
            }
            _ => panic!("expected Part::File"),
        }

        let data_part = Part::Data {
            data: serde_json::json!({"key": "value", "count": 42}),
        };
        let json = serde_json::to_string(&data_part).expect("serialize");
        let deserialized: Part = serde_json::from_str(&json).expect("deserialize");
        match deserialized {
            Part::Data { data } => assert_eq!(data["key"], "value"),
            _ => panic!("expected Part::Data"),
        }
    }

    #[test]
    fn a2a_message_roundtrip() {
        let msg = A2AMessage {
            role: "user".to_string(),
            parts: vec![
                Part::Text { text: "Hello".to_string() },
                Part::Data { data: serde_json::json!({"intent": "greet"}) },
            ],
        };
        let json = serde_json::to_string(&msg).expect("serialize");
        let deserialized: A2AMessage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.role, "user");
        assert_eq!(deserialized.parts.len(), 2);
    }

    #[test]
    fn a2a_task_roundtrip() {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("priority".to_string(), "high".to_string());

        let task = A2ATask {
            id: "task-001".to_string(),
            state: TaskState::Working,
            messages: vec![A2AMessage {
                role: "agent".to_string(),
                parts: vec![Part::Text { text: "Processing".to_string() }],
            }],
            artifacts: Some(vec![Part::File {
                name: "output.json".to_string(),
                mime_type: "application/json".to_string(),
                data: "e30=".to_string(),
            }]),
            metadata: Some(metadata),
        };
        let json = serde_json::to_string(&task).expect("serialize");
        let deserialized: A2ATask = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.id, "task-001");
        assert_eq!(deserialized.state, TaskState::Working);
        assert_eq!(deserialized.messages.len(), 1);
        assert!(deserialized.artifacts.is_some());
        assert_eq!(deserialized.metadata.unwrap()["priority"], "high");

        // Verify None fields are omitted
        let task_minimal = A2ATask {
            id: "task-002".to_string(),
            state: TaskState::Submitted,
            messages: vec![],
            artifacts: None,
            metadata: None,
        };
        let json_minimal = serde_json::to_string(&task_minimal).expect("serialize");
        assert!(!json_minimal.contains("artifacts"));
        assert!(!json_minimal.contains("metadata"));
    }
}
