use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::cluster::SwarmCluster;
use crate::consensus::{ConsensusManager, GossipLayer, PbftLayer, RaftLayer};
use crate::health::HealthMonitor;
use crate::node::SwarmNode;
use crate::types::*;

/// Status of the orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorStatus {
    pub cluster_id: ClusterId,
    pub running: bool,
    pub node_count: usize,
    pub healthy_count: usize,
    pub zone_count: usize,
    #[serde(with = "duration_millis")]
    pub uptime: Duration,
}

/// Top-level entry point for managing a swarm.
pub struct SwarmOrchestrator {
    pub cluster: SwarmCluster,
    pub consensus: ConsensusManager,
    pub health: HealthMonitor,
    pub config: SwarmConfig,
    running: bool,
    started_at: Option<Instant>,
}

impl SwarmOrchestrator {
    /// Create a new orchestrator from the given config.
    pub fn new(config: SwarmConfig) -> Self {
        let cluster = SwarmCluster::new(config.clone());
        let health = HealthMonitor::new(config.health_interval, 3);

        // Build consensus manager with default layers.
        let mut consensus = ConsensusManager::new();
        consensus.register(Box::new(PbftLayer::new(Vec::new())));
        consensus.register(Box::new(RaftLayer::new(Vec::new())));
        consensus.register(Box::new(GossipLayer::new(Vec::new(), 3)));

        Self {
            cluster,
            consensus,
            health,
            config,
            running: false,
            started_at: None,
        }
    }

    /// Start the orchestrator.
    pub async fn start(&mut self) {
        self.running = true;
        self.started_at = Some(Instant::now());
        tracing::info!(cluster_id = %self.cluster.id.0, "orchestrator started");
    }

    /// Stop the orchestrator.
    pub async fn stop(&mut self) {
        self.running = false;
        tracing::info!(cluster_id = %self.cluster.id.0, "orchestrator stopped");
    }

    /// Add a node to a zone by creating it from a profile.
    pub fn add_node(
        &mut self,
        zone_id: &ZoneId,
        profile: NodeProfile,
    ) -> Result<NodeId, SwarmError> {
        let mut node = SwarmNode::new(zone_id.clone(), profile, "auto");
        node.update_heartbeat();
        self.cluster.add_node(zone_id, node)
    }

    /// Remove a node by ID.
    pub fn remove_node(&mut self, node_id: &NodeId) -> Result<(), SwarmError> {
        self.cluster.remove_node(node_id)?;
        Ok(())
    }

    /// Get the current cluster topology.
    pub fn topology(&self) -> crate::cluster::SwarmTopology {
        self.cluster.topology()
    }

    /// Route a syscall to the appropriate zone based on strategy.
    ///
    /// Returns the first zone ID from the strategy's placement mapping that
    /// contains at least one node, or `None` if no suitable zone is available.
    pub fn route_syscall(&self, strategy: &str) -> Option<String> {
        let mapping = StrategyZoneMapping::default_mapping();
        let zones = mapping.zones_for_strategy(strategy);
        for zone_id_str in &zones {
            let zone_id = ZoneId::new(zone_id_str.as_str());
            if let Some(zone) = self.cluster.zones.get_zone(&zone_id) {
                if !zone.nodes.is_empty() {
                    return Some(zone_id_str.clone());
                }
            }
        }
        None
    }

    /// Get orchestrator status.
    pub fn status(&self) -> OrchestratorStatus {
        let uptime = self
            .started_at
            .map(|s| s.elapsed())
            .unwrap_or(Duration::ZERO);
        OrchestratorStatus {
            cluster_id: self.cluster.id,
            running: self.running,
            node_count: self.cluster.node_count(),
            healthy_count: self.cluster.healthy_node_count(),
            zone_count: self.cluster.zones.zones.len(),
            uptime,
        }
    }
}

/// Serde helper for Duration as milliseconds.
mod duration_millis {
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
    use std::collections::HashMap;

    fn test_config() -> SwarmConfig {
        SwarmConfig {
            cluster_id: ClusterId::new(),
            zones: vec![
                ZoneConfig {
                    id: ZoneId::new("z1"),
                    name: "Compute".into(),
                    consensus_type: ConsensusType::Pbft,
                    placement_policy: PlacementPolicy::ComputeHeavy,
                },
                ZoneConfig {
                    id: ZoneId::new("z2"),
                    name: "Edge".into(),
                    consensus_type: ConsensusType::Gossip,
                    placement_policy: PlacementPolicy::EdgeRelay,
                },
            ],
            max_nodes: 50,
            consensus: ConsensusConfig {
                zone_configs: HashMap::new(),
                default_type: ConsensusType::Raft,
            },
            health_interval: Duration::from_secs(5),
        }
    }

    #[tokio::test]
    async fn test_orchestrator_start_stop() {
        let mut orch = SwarmOrchestrator::new(test_config());
        assert!(!orch.status().running);
        orch.start().await;
        assert!(orch.status().running);
        orch.stop().await;
        assert!(!orch.status().running);
    }

    #[test]
    fn test_add_node() {
        let mut orch = SwarmOrchestrator::new(test_config());
        let profile = NodeProfile {
            cpu_cores: 8,
            memory_mb: 16384,
            has_gpu: false,
            gpu_type: None,
            architecture: "x86_64".into(),
        };
        let id = orch.add_node(&ZoneId::new("z1"), profile).unwrap();
        assert_eq!(orch.status().node_count, 1);
        assert!(orch.cluster.zones.get_node(&id).is_some());
    }

    #[test]
    fn test_remove_node() {
        let mut orch = SwarmOrchestrator::new(test_config());
        let profile = NodeProfile {
            cpu_cores: 4,
            memory_mb: 4096,
            has_gpu: false,
            gpu_type: None,
            architecture: "aarch64".into(),
        };
        let id = orch.add_node(&ZoneId::new("z2"), profile).unwrap();
        orch.remove_node(&id).unwrap();
        assert_eq!(orch.status().node_count, 0);
    }

    #[test]
    fn test_topology() {
        let mut orch = SwarmOrchestrator::new(test_config());
        let topo = orch.topology();
        assert_eq!(topo.zones.len(), 2);

        let profile = NodeProfile {
            cpu_cores: 8,
            memory_mb: 16384,
            has_gpu: false,
            gpu_type: None,
            architecture: "x86_64".into(),
        };
        orch.add_node(&ZoneId::new("z1"), profile).unwrap();
        let topo = orch.topology();
        assert_eq!(topo.total_nodes, 1);
    }

    #[test]
    fn test_status_zone_count() {
        let orch = SwarmOrchestrator::new(test_config());
        assert_eq!(orch.status().zone_count, 2);
    }

    /// Config with ADR-001 zone IDs for route_syscall tests.
    fn adr_config() -> SwarmConfig {
        SwarmConfig {
            cluster_id: ClusterId::new(),
            zones: vec![
                ZoneConfig {
                    id: ZoneId::new("zone-a"),
                    name: "Compute".into(),
                    consensus_type: ConsensusType::Pbft,
                    placement_policy: PlacementPolicy::ComputeHeavy,
                },
                ZoneConfig {
                    id: ZoneId::new("zone-b"),
                    name: "Inference".into(),
                    consensus_type: ConsensusType::Raft,
                    placement_policy: PlacementPolicy::InferenceOptimized,
                },
                ZoneConfig {
                    id: ZoneId::new("zone-c"),
                    name: "Edge".into(),
                    consensus_type: ConsensusType::Gossip,
                    placement_policy: PlacementPolicy::EdgeRelay,
                },
                ZoneConfig {
                    id: ZoneId::new("zone-d"),
                    name: "Burst".into(),
                    consensus_type: ConsensusType::Raft,
                    placement_policy: PlacementPolicy::Any,
                },
            ],
            max_nodes: 50,
            consensus: ConsensusConfig {
                zone_configs: HashMap::new(),
                default_type: ConsensusType::Raft,
            },
            health_interval: Duration::from_secs(5),
        }
    }

    fn gpu_profile() -> NodeProfile {
        NodeProfile {
            cpu_cores: 16,
            memory_mb: 32768,
            has_gpu: true,
            gpu_type: Some("cuda".into()),
            architecture: "x86_64".into(),
        }
    }

    #[test]
    fn test_route_syscall_returns_healthy_zone() {
        let mut orch = SwarmOrchestrator::new(adr_config());
        orch.add_node(&ZoneId::new("zone-a"), gpu_profile())
            .unwrap();

        let routed = orch.route_syscall("Rlm");
        assert_eq!(routed, Some("zone-a".into()));
    }

    #[test]
    fn test_route_syscall_falls_back_to_secondary() {
        let mut orch = SwarmOrchestrator::new(adr_config());
        // zone-a is empty, zone-d has a node => Rlm should fall back to zone-d
        orch.add_node(&ZoneId::new("zone-d"), gpu_profile())
            .unwrap();

        let routed = orch.route_syscall("Rlm");
        assert_eq!(routed, Some("zone-d".into()));
    }

    #[test]
    fn test_route_syscall_returns_none_when_no_nodes() {
        let orch = SwarmOrchestrator::new(adr_config());
        let routed = orch.route_syscall("Rlm");
        assert!(routed.is_none());
    }

    #[test]
    fn test_route_syscall_unknown_strategy() {
        let orch = SwarmOrchestrator::new(adr_config());
        let routed = orch.route_syscall("UnknownStrategy");
        assert!(routed.is_none());
    }

    #[test]
    fn test_route_syscall_edge_prefers_zone_b() {
        let mut orch = SwarmOrchestrator::new(adr_config());
        orch.add_node(&ZoneId::new("zone-b"), gpu_profile())
            .unwrap();
        orch.add_node(&ZoneId::new("zone-c"), gpu_profile())
            .unwrap();

        let routed = orch.route_syscall("Edge");
        assert_eq!(
            routed,
            Some("zone-b".into()),
            "Edge should prefer zone-b as primary"
        );
    }
}
