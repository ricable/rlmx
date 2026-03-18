use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::chaos::FaultType;
use crate::cluster::SwarmCluster;
use crate::health::HealthMonitor;
use crate::node::SwarmNode;
use crate::transport::InMemoryTransport;
use crate::types::*;
use std::collections::HashMap;

/// Status snapshot of a simulated swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmStatus {
    pub nodes: usize,
    pub healthy: usize,
    pub zones: usize,
    pub uptime_secs: u64,
}

/// Configurable latency parameters for simulation, matching ADR-001 zone budgets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyConfig {
    /// Per-zone intra-zone latency in milliseconds.
    pub intra_zone_ms: HashMap<String, u64>,
    /// Default cross-zone latency in milliseconds.
    pub inter_zone_ms: u64,
}

impl Default for LatencyConfig {
    fn default() -> Self {
        let mut intra = HashMap::new();
        intra.insert("zone-a".into(), 5); // <50ms compute zone
        intra.insert("zone-b".into(), 10); // <100ms inference zone
        intra.insert("zone-c".into(), 20); // <200ms edge zone
        intra.insert("zone-d".into(), 50); // variable burst zone
        Self {
            intra_zone_ms: intra,
            inter_zone_ms: 100,
        }
    }
}

impl LatencyConfig {
    /// Get the simulated latency between two zones.
    pub fn latency_between(&self, from: &str, to: &str) -> u64 {
        if from == to {
            self.intra_zone_ms.get(from).copied().unwrap_or(10)
        } else {
            self.inter_zone_ms
        }
    }
}

/// A simulated swarm for testing and development.
pub struct SimulatedSwarm {
    pub cluster: SwarmCluster,
    pub transport: InMemoryTransport,
    pub health: HealthMonitor,
    pub latency: LatencyConfig,
    pub running: bool,
    started_at: Option<std::time::Instant>,
}

impl SimulatedSwarm {
    /// Create a simulated swarm with `node_count` nodes across 3 zones.
    /// Zone A = compute, Zone B = inference, Zone C = edge.
    pub fn new(node_count: usize) -> Self {
        let config = SwarmConfig {
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
            max_nodes: node_count.max(100),
            consensus: ConsensusConfig {
                zone_configs: HashMap::new(),
                default_type: ConsensusType::Raft,
            },
            health_interval: Duration::from_secs(5),
        };

        let mut cluster = SwarmCluster::new(config);
        let zone_ids = ["zone-a", "zone-b", "zone-c", "zone-d"];

        for i in 0..node_count {
            let zone_idx = i % 4;
            let zone_id = ZoneId::new(zone_ids[zone_idx]);
            let profile = match zone_idx {
                0 => NodeProfile {
                    cpu_cores: 16,
                    memory_mb: 32768,
                    has_gpu: false,
                    gpu_type: None,
                    architecture: "x86_64".into(),
                },
                1 => NodeProfile {
                    cpu_cores: 8,
                    memory_mb: 16384,
                    has_gpu: true,
                    gpu_type: Some("cuda".into()),
                    architecture: "x86_64".into(),
                },
                2 => NodeProfile {
                    cpu_cores: 4,
                    memory_mb: 4096,
                    has_gpu: false,
                    gpu_type: None,
                    architecture: "aarch64".into(),
                },
                _ => NodeProfile {
                    cpu_cores: 8,
                    memory_mb: 32768,
                    has_gpu: true,
                    gpu_type: Some("cuda".into()),
                    architecture: "x86_64".into(),
                },
            };
            let mut node = SwarmNode::new(zone_id.clone(), profile, format!("sim-node-{i}:9000"));
            node.update_heartbeat();
            let _ = cluster.add_node(&zone_id, node);
        }

        let transport = InMemoryTransport::new(256);
        let health = HealthMonitor::new(Duration::from_secs(5), 3);

        Self {
            cluster,
            transport,
            health,
            latency: LatencyConfig::default(),
            running: false,
            started_at: None,
        }
    }

    /// Start the simulated swarm (marks it running).
    pub async fn start(&mut self) {
        self.running = true;
        self.started_at = Some(std::time::Instant::now());
        tracing::info!(nodes = self.cluster.node_count(), "simulated swarm started");
    }

    /// Stop the simulated swarm.
    pub async fn stop(&mut self) {
        self.running = false;
        tracing::info!("simulated swarm stopped");
    }

    /// Inject a fault for chaos testing.
    pub fn inject_fault(&mut self, fault: FaultType) {
        match &fault {
            FaultType::NodeCrash(node_id) => {
                let _ = self.cluster.remove_node(node_id);
                tracing::warn!(%node_id, "simulated node crash");
            }
            FaultType::ZoneFailure(zone_id) => {
                if let Some(zone) = self.cluster.zones.zones.get(zone_id) {
                    let node_ids: Vec<NodeId> = zone.nodes.keys().copied().collect();
                    for nid in node_ids {
                        let _ = self.cluster.remove_node(&nid);
                    }
                }
                tracing::warn!(%zone_id, "simulated zone failure");
            }
            _ => {
                tracing::info!(?fault, "fault injection recorded (simulation only)");
            }
        }
    }

    /// Get a status snapshot.
    pub fn status(&self) -> SwarmStatus {
        let uptime = self.started_at.map(|s| s.elapsed().as_secs()).unwrap_or(0);
        SwarmStatus {
            nodes: self.cluster.node_count(),
            healthy: self.cluster.healthy_node_count(),
            zones: self.cluster.zones.zones.len(),
            uptime_secs: uptime,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_4_node_swarm() {
        let swarm = SimulatedSwarm::new(4);
        assert_eq!(swarm.cluster.node_count(), 4);
        assert_eq!(swarm.cluster.zones.zones.len(), 4);
    }

    #[tokio::test]
    async fn test_start_stop() {
        let mut swarm = SimulatedSwarm::new(4);
        assert!(!swarm.running);
        swarm.start().await;
        assert!(swarm.running);
        swarm.stop().await;
        assert!(!swarm.running);
    }

    #[test]
    fn test_health_monitoring() {
        let swarm = SimulatedSwarm::new(8);
        let status = swarm.status();
        assert_eq!(status.nodes, 8);
        assert_eq!(status.healthy, 8);
        assert_eq!(status.zones, 4);
    }

    #[test]
    fn test_zone_d_exists_in_simulation() {
        let swarm = SimulatedSwarm::new(4);
        let zone_d = swarm.cluster.zones.get_zone(&ZoneId::new("zone-d"));
        assert!(zone_d.is_some(), "Zone D (Burst) must exist in simulation");
        let zone = zone_d.unwrap();
        assert_eq!(zone.name, "Burst");
        assert_eq!(zone.consensus_type, ConsensusType::Raft);
        assert_eq!(zone.policy, PlacementPolicy::Any);
        assert!(
            !zone.nodes.is_empty(),
            "Zone D should have at least one node"
        );
    }

    #[test]
    fn test_all_four_zones_present() {
        let swarm = SimulatedSwarm::new(8);
        for (id, expected_name) in [
            ("zone-a", "Compute"),
            ("zone-b", "Inference"),
            ("zone-c", "Edge"),
            ("zone-d", "Burst"),
        ] {
            let zone = swarm.cluster.zones.get_zone(&ZoneId::new(id));
            assert!(zone.is_some(), "Zone {id} must exist");
            assert_eq!(zone.unwrap().name, expected_name);
        }
    }

    #[test]
    fn test_latency_config_defaults_match_adr() {
        let latency = LatencyConfig::default();
        assert_eq!(latency.intra_zone_ms["zone-a"], 5); // <50ms compute
        assert_eq!(latency.intra_zone_ms["zone-b"], 10); // <100ms inference
        assert_eq!(latency.intra_zone_ms["zone-c"], 20); // <200ms edge
        assert_eq!(latency.intra_zone_ms["zone-d"], 50); // variable burst
        assert_eq!(latency.inter_zone_ms, 100);
    }

    #[test]
    fn test_latency_config_intra_vs_inter() {
        let latency = LatencyConfig::default();
        // Same zone should return intra-zone latency
        assert_eq!(latency.latency_between("zone-a", "zone-a"), 5);
        // Different zones should return inter-zone latency
        assert_eq!(latency.latency_between("zone-a", "zone-b"), 100);
    }

    #[test]
    fn test_simulated_swarm_has_latency_config() {
        let swarm = SimulatedSwarm::new(4);
        assert_eq!(swarm.latency.intra_zone_ms.len(), 4);
        assert_eq!(swarm.latency.inter_zone_ms, 100);
    }

    #[test]
    fn test_inject_node_crash() {
        let mut swarm = SimulatedSwarm::new(6);
        let node_id = *swarm
            .cluster
            .zones
            .zones
            .values()
            .next()
            .unwrap()
            .nodes
            .keys()
            .next()
            .unwrap();
        swarm.inject_fault(FaultType::NodeCrash(node_id));
        assert_eq!(swarm.cluster.node_count(), 5);
    }

    #[test]
    fn test_status_uptime() {
        let swarm = SimulatedSwarm::new(3);
        let status = swarm.status();
        assert_eq!(status.uptime_secs, 0);
    }
}
