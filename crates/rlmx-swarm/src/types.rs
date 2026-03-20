use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// Unique identifier for a swarm node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier for a zone within the swarm.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZoneId(pub String);

impl ZoneId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for ZoneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClusterId(pub Uuid);

impl ClusterId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ClusterId {
    fn default() -> Self {
        Self::new()
    }
}

/// Hardware profile of a swarm node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeProfile {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub has_gpu: bool,
    pub gpu_type: Option<String>,
    pub architecture: String,
}

/// Configuration for the entire swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmConfig {
    pub cluster_id: ClusterId,
    pub zones: Vec<ZoneConfig>,
    pub max_nodes: usize,
    pub consensus: ConsensusConfig,
    #[serde(with = "duration_serde")]
    pub health_interval: Duration,
}

/// Configuration for a single zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneConfig {
    pub id: ZoneId,
    pub name: String,
    pub consensus_type: ConsensusType,
    pub placement_policy: PlacementPolicy,
}

/// Type of consensus protocol used within a zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsensusType {
    Pbft,
    Raft,
    Gossip,
    QuDag,
}

/// Consensus configuration mapping zones to consensus types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub zone_configs: HashMap<ZoneId, ConsensusType>,
    pub default_type: ConsensusType,
}

/// Node placement policy for a zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlacementPolicy {
    ComputeHeavy,
    InferenceOptimized,
    EdgeRelay,
    /// Browser WASM compute workers (ADR-009). Stateless, no consensus participation.
    BrowserCompute,
    Any,
}

/// Events emitted by the swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwarmEvent {
    NodeJoined {
        node_id: NodeId,
        zone_id: ZoneId,
    },
    NodeLeft {
        node_id: NodeId,
        zone_id: ZoneId,
    },
    HealthUpdate {
        node_id: NodeId,
        status: String,
    },
    AgentSpawned {
        agent_id: String,
        node_id: NodeId,
    },
    AgentTerminated {
        agent_id: String,
        node_id: NodeId,
    },
    ConsensusReached {
        round: u64,
        value_hash: String,
    },
    ExperimentUpdate {
        experiment_id: String,
        progress: f64,
    },
    MutationFound {
        experiment_id: String,
        description: String,
    },
    SandboxSpawned {
        sandbox_id: String,
        profile: String,
        node_id: Option<String>,
    },
    SandboxTerminated {
        sandbox_id: String,
        reason: String,
    },
    /// Real-time voice transcription chunk (ADR-018).
    VoiceChunk {
        session_id: String,
        transcript: String,
        confidence: f32,
        is_final: bool,
    },
    /// Coordination board update (ADR-031).
    BoardUpdate {
        board_id: String,
        post_id: String,
        author: String,
        action: String,
    },
    /// Human approval required for an operation (ADR-037).
    ApprovalRequired {
        request_id: String,
        operation: String,
        agent_id: String,
        tier: String,
        cost_estimate: Option<u64>,
    },
    /// Per-agent progress update for multi-intent fan-out (ADR-015/018).
    AgentProgress {
        task_id: String,
        agent_type: String,
        domain: String,
        progress_pct: f32,
        status_text: String,
        eta_ms: Option<u64>,
    },
    /// Multimodal response combining voice, visual, and haptic channels (ADR-018).
    MultimodalResponse {
        session_id: String,
        voice_text: Option<String>,
        visual_card: Option<serde_json::Value>,
        haptic_pattern: Option<String>,
        is_final: bool,
    },
}

/// Structured visual card data for multimodal responses (ADR-018).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardData {
    pub card_type: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub body: Option<String>,
    pub data: Option<serde_json::Value>,
    pub actions: Vec<String>,
    pub domain: String,
}

/// Haptic feedback pattern for multimodal responses (ADR-018).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HapticPattern {
    Gentle,
    DoubleTap,
    LongBuzz,
    Alert,
    Success,
    Warning,
}

/// Maps kernel `Strategy` names to their preferred zone placement order.
///
/// Each strategy maps to a list of zone IDs where the first is the primary zone
/// and subsequent entries are fallback zones, matching ADR-001's placement policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyZoneMapping {
    mappings: HashMap<String, Vec<String>>,
}

impl StrategyZoneMapping {
    /// Create the default mapping per ADR-001 placement policy table.
    pub fn default_mapping() -> Self {
        let mut m = HashMap::new();
        m.insert("Rlm".into(), vec!["zone-a".into(), "zone-d".into()]);
        m.insert("Trm".into(), vec!["zone-a".into(), "zone-b".into()]);
        m.insert("Edge".into(), vec!["zone-b".into(), "zone-c".into()]);
        m.insert("Hybrid".into(), vec!["zone-a".into()]);
        m.insert(
            "Swarm".into(),
            vec!["zone-a".into(), "zone-b".into(), "zone-c".into()],
        );
        Self { mappings: m }
    }

    /// Return the ordered list of zone IDs for the given strategy.
    pub fn zones_for_strategy(&self, strategy: &str) -> Vec<String> {
        self.mappings.get(strategy).cloned().unwrap_or_default()
    }
}

/// Health and performance metrics update disseminated via gossip.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsUpdate {
    pub node: NodeId,
    pub timestamp: DateTime<Utc>,
    pub load: f32,
    pub memory_pressure: f32,
    pub inference_latency_p99_ms: u64,
    pub active_processes: usize,
    pub vector_segments: usize,
}

/// Errors produced by swarm operations.
#[derive(Debug, thiserror::Error)]
pub enum SwarmError {
    #[error("node not found: {0}")]
    NodeNotFound(NodeId),
    #[error("zone not found: {0}")]
    ZoneNotFound(ZoneId),
    #[error("cluster full: max {max} nodes")]
    ClusterFull { max: usize },
    #[error("consensus failed: {0}")]
    ConsensusFailed(String),
    #[error("transport error: {0}")]
    TransportError(String),
    #[error("health check failed: {0}")]
    HealthCheckFailed(String),
    #[error("cloud provider error: {0}")]
    CloudError(String),
    #[error("not configured: {0}")]
    NotConfigured(String),
    #[error("invalid operation: {0}")]
    InvalidOperation(String),
}

/// Serde helper for Duration (stored as milliseconds).
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        d.as_millis().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let ms = u64::deserialize(d)?;
        Ok(Duration::from_millis(ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_unique() {
        let a = NodeId::new();
        let b = NodeId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn test_zone_id() {
        let z = ZoneId::new("zone-a");
        assert_eq!(z.0, "zone-a");
    }

    #[test]
    fn test_cluster_id_default() {
        let c = ClusterId::default();
        let d = ClusterId::default();
        assert_ne!(c, d);
    }

    #[test]
    fn test_swarm_event_serialize() {
        let event = SwarmEvent::NodeJoined {
            node_id: NodeId::new(),
            zone_id: ZoneId::new("z1"),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("NodeJoined"));
    }

    #[test]
    fn test_consensus_type_eq() {
        assert_eq!(ConsensusType::Pbft, ConsensusType::Pbft);
        assert_ne!(ConsensusType::Raft, ConsensusType::Gossip);
    }

    #[test]
    fn test_swarm_error_display() {
        let err = SwarmError::ClusterFull { max: 10 };
        assert_eq!(err.to_string(), "cluster full: max 10 nodes");
    }

    #[test]
    fn test_metrics_update_serialize() {
        let update = MetricsUpdate {
            node: NodeId::new(),
            timestamp: Utc::now(),
            load: 0.75,
            memory_pressure: 0.3,
            inference_latency_p99_ms: 42,
            active_processes: 5,
            vector_segments: 12,
        };
        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("\"load\":0.75"));
        let roundtrip: MetricsUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.active_processes, 5);
        assert_eq!(roundtrip.vector_segments, 12);
    }

    #[test]
    fn test_strategy_zone_mapping_rlm() {
        let mapping = StrategyZoneMapping::default_mapping();
        let zones = mapping.zones_for_strategy("Rlm");
        assert_eq!(zones, vec!["zone-a", "zone-d"]);
    }

    #[test]
    fn test_strategy_zone_mapping_edge_with_fallback() {
        let mapping = StrategyZoneMapping::default_mapping();
        let zones = mapping.zones_for_strategy("Edge");
        assert_eq!(zones.len(), 2);
        assert_eq!(zones[0], "zone-b"); // primary
        assert_eq!(zones[1], "zone-c"); // fallback
    }

    #[test]
    fn test_strategy_zone_mapping_swarm_cross_zone() {
        let mapping = StrategyZoneMapping::default_mapping();
        let zones = mapping.zones_for_strategy("Swarm");
        assert_eq!(zones, vec!["zone-a", "zone-b", "zone-c"]);
    }

    #[test]
    fn test_strategy_zone_mapping_unknown_returns_empty() {
        let mapping = StrategyZoneMapping::default_mapping();
        let zones = mapping.zones_for_strategy("UnknownStrategy");
        assert!(zones.is_empty());
    }

    #[test]
    fn test_strategy_zone_mapping_all_strategies_present() {
        let mapping = StrategyZoneMapping::default_mapping();
        for strategy in &["Rlm", "Trm", "Edge", "Hybrid", "Swarm"] {
            let zones = mapping.zones_for_strategy(strategy);
            assert!(
                !zones.is_empty(),
                "{strategy} should have at least one zone"
            );
        }
    }
}
