use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::node::SwarmNode;
use crate::types::{ClusterId, ConsensusType, NodeId, SwarmConfig, SwarmError, SwarmEvent, ZoneId};
use crate::zone::{Zone, ZoneManager};

/// The top-level swarm cluster, managing zones and broadcasting events.
pub struct SwarmCluster {
    pub id: ClusterId,
    pub config: SwarmConfig,
    pub zones: ZoneManager,
    pub event_tx: broadcast::Sender<SwarmEvent>,
}

impl SwarmCluster {
    /// Create a new cluster from the given config.
    pub fn new(config: SwarmConfig) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        let mut zones = ZoneManager::new();

        for zc in &config.zones {
            zones.add_zone(Zone::new(
                zc.id.clone(),
                &zc.name,
                zc.consensus_type,
                zc.placement_policy,
            ));
        }

        Self {
            id: config.cluster_id,
            config,
            zones,
            event_tx,
        }
    }

    /// Add a node to the specified zone.
    pub fn add_node(&mut self, zone_id: &ZoneId, node: SwarmNode) -> Result<NodeId, SwarmError> {
        if self.zones.total_nodes() >= self.config.max_nodes {
            return Err(SwarmError::ClusterFull {
                max: self.config.max_nodes,
            });
        }
        let node_id = node.id;
        self.zones.add_node(zone_id, node)?;
        let _ = self.event_tx.send(SwarmEvent::NodeJoined {
            node_id,
            zone_id: zone_id.clone(),
        });
        tracing::info!(%node_id, %zone_id, "node joined cluster");
        Ok(node_id)
    }

    /// Remove a node from the cluster.
    pub fn remove_node(&mut self, node_id: &NodeId) -> Result<SwarmNode, SwarmError> {
        let node = self.zones.remove_node(node_id)?;
        let _ = self.event_tx.send(SwarmEvent::NodeLeft {
            node_id: *node_id,
            zone_id: node.zone.clone(),
        });
        tracing::info!(%node_id, "node left cluster");
        Ok(node)
    }

    /// Get a snapshot of the cluster topology.
    pub fn topology(&self) -> SwarmTopology {
        let zone_infos: Vec<ZoneInfo> = self
            .zones
            .zones
            .values()
            .map(|z| ZoneInfo {
                id: z.id.clone(),
                name: z.name.clone(),
                node_count: z.nodes.len(),
                consensus_type: z.consensus_type,
            })
            .collect();

        // Build connections: fully connected mesh between zones.
        let zone_ids: Vec<&ZoneId> = self.zones.zones.keys().collect();
        let mut connections = Vec::new();
        for (i, from) in zone_ids.iter().enumerate() {
            for to in zone_ids.iter().skip(i + 1) {
                connections.push(Connection {
                    from: (*from).clone(),
                    to: (*to).clone(),
                    latency_ms: 1, // simulated
                });
            }
        }

        let total = self.zones.total_nodes();
        let healthy = self
            .zones
            .zones
            .keys()
            .map(|zid| self.zones.healthy_nodes(zid).len())
            .sum();

        SwarmTopology {
            zones: zone_infos,
            total_nodes: total,
            healthy_nodes: healthy,
            connections,
        }
    }

    /// Broadcast an event to all subscribers.
    pub fn broadcast_event(&self, event: SwarmEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Total number of nodes in the cluster.
    pub fn node_count(&self) -> usize {
        self.zones.total_nodes()
    }

    /// Number of healthy nodes in the cluster.
    pub fn healthy_node_count(&self) -> usize {
        self.zones
            .zones
            .keys()
            .map(|zid| self.zones.healthy_nodes(zid).len())
            .sum()
    }
}

/// Serializable snapshot of the cluster topology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmTopology {
    pub zones: Vec<ZoneInfo>,
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub connections: Vec<Connection>,
}

/// Info about a single zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneInfo {
    pub id: ZoneId,
    pub name: String,
    pub node_count: usize,
    pub consensus_type: ConsensusType,
}

/// A connection between two zones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from: ZoneId,
    pub to: ZoneId,
    pub latency_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::collections::HashMap;
    use std::time::Duration;

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
                    name: "Inference".into(),
                    consensus_type: ConsensusType::Raft,
                    placement_policy: PlacementPolicy::InferenceOptimized,
                },
            ],
            max_nodes: 10,
            consensus: ConsensusConfig {
                zone_configs: HashMap::new(),
                default_type: ConsensusType::Raft,
            },
            health_interval: Duration::from_secs(5),
        }
    }

    fn make_node(zone_id: &str) -> SwarmNode {
        SwarmNode::new(
            ZoneId::new(zone_id),
            NodeProfile {
                cpu_cores: 8,
                memory_mb: 16384,
                has_gpu: false,
                gpu_type: None,
                architecture: "x86_64".into(),
            },
            "127.0.0.1:9000",
        )
    }

    #[test]
    fn test_new_cluster_has_zones() {
        let cluster = SwarmCluster::new(test_config());
        assert_eq!(cluster.zones.zones.len(), 2);
    }

    #[test]
    fn test_add_node() {
        let mut cluster = SwarmCluster::new(test_config());
        let node = make_node("z1");
        let id = cluster.add_node(&ZoneId::new("z1"), node).unwrap();
        assert_eq!(cluster.node_count(), 1);
        assert!(cluster.zones.get_node(&id).is_some());
    }

    #[test]
    fn test_add_node_cluster_full() {
        let mut config = test_config();
        config.max_nodes = 1;
        let mut cluster = SwarmCluster::new(config);
        cluster
            .add_node(&ZoneId::new("z1"), make_node("z1"))
            .unwrap();
        let result = cluster.add_node(&ZoneId::new("z1"), make_node("z1"));
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_node() {
        let mut cluster = SwarmCluster::new(test_config());
        let node = make_node("z1");
        let id = cluster.add_node(&ZoneId::new("z1"), node).unwrap();
        let removed = cluster.remove_node(&id).unwrap();
        assert_eq!(removed.id, id);
        assert_eq!(cluster.node_count(), 0);
    }

    #[test]
    fn test_topology() {
        let mut cluster = SwarmCluster::new(test_config());
        cluster
            .add_node(&ZoneId::new("z1"), make_node("z1"))
            .unwrap();
        let topo = cluster.topology();
        assert_eq!(topo.zones.len(), 2);
        assert_eq!(topo.total_nodes, 1);
        assert_eq!(topo.connections.len(), 1); // 2 zones => 1 connection
    }

    #[test]
    fn test_event_broadcast() {
        let cluster = SwarmCluster::new(test_config());
        let mut rx = cluster.event_tx.subscribe();
        cluster.broadcast_event(SwarmEvent::ConsensusReached {
            round: 1,
            value_hash: "abc".into(),
        });
        let event = rx.try_recv().unwrap();
        match event {
            SwarmEvent::ConsensusReached { round, .. } => assert_eq!(round, 1),
            _ => panic!("unexpected event"),
        }
    }
}
