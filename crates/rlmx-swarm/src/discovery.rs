//! Node discovery for swarm membership.
//!
//! When the `ruv-swarm` feature is enabled, provides gossip-based peer
//! discovery backed by `ruv-swarm-core` topology primitives.

use crate::types::{NodeId, ZoneId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Metadata announced by a node during discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAnnouncement {
    pub node_id: NodeId,
    pub zone_id: ZoneId,
    pub address: String,
    pub capabilities: Vec<String>,
    pub announced_at: DateTime<Utc>,
}

/// A discovered peer with last-seen tracking.
#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    pub announcement: NodeAnnouncement,
    pub last_seen: DateTime<Utc>,
    pub alive: bool,
}

/// Gossip-based swarm discovery service.
///
/// Nodes announce their presence and discover peers through periodic
/// gossip rounds. When the `ruv-swarm` feature is active, the topology
/// graph from `ruv-swarm-core` is used for neighbor selection.
pub struct SwarmDiscovery {
    local_node: NodeId,
    local_zone: ZoneId,
    peers: Arc<Mutex<HashMap<NodeId, DiscoveredPeer>>>,
    ttl_ms: u64,
}

impl SwarmDiscovery {
    /// Create a new discovery service for the given local node.
    pub fn new(local_node: NodeId, local_zone: ZoneId) -> Self {
        Self {
            local_node,
            local_zone,
            peers: Arc::new(Mutex::new(HashMap::new())),
            ttl_ms: 30_000,
        }
    }

    /// Set the TTL (in ms) after which a peer is considered stale.
    pub fn with_ttl_ms(mut self, ttl_ms: u64) -> Self {
        self.ttl_ms = ttl_ms;
        self
    }

    /// Announce our presence by producing an announcement record.
    pub fn announce(&self, address: String, capabilities: Vec<String>) -> NodeAnnouncement {
        NodeAnnouncement {
            node_id: self.local_node,
            zone_id: self.local_zone.clone(),
            address,
            capabilities,
            announced_at: Utc::now(),
        }
    }

    /// Process an incoming announcement from a peer.
    pub fn process_announcement(&self, announcement: NodeAnnouncement) {
        // Ignore self-announcements.
        if announcement.node_id == self.local_node {
            return;
        }
        let mut peers = self.peers.lock().unwrap();
        peers.insert(
            announcement.node_id,
            DiscoveredPeer {
                announcement,
                last_seen: Utc::now(),
                alive: true,
            },
        );
    }

    /// Discover all currently-known live peers.
    pub fn discover_peers(&self) -> Vec<NodeAnnouncement> {
        let peers = self.peers.lock().unwrap();
        peers
            .values()
            .filter(|p| p.alive)
            .map(|p| p.announcement.clone())
            .collect()
    }

    /// Discover peers in a specific zone.
    pub fn discover_peers_in_zone(&self, zone: &ZoneId) -> Vec<NodeAnnouncement> {
        let peers = self.peers.lock().unwrap();
        peers
            .values()
            .filter(|p| p.alive && p.announcement.zone_id == *zone)
            .map(|p| p.announcement.clone())
            .collect()
    }

    /// Simulate a join by processing multiple announcements at once.
    pub fn join(&self, announcements: Vec<NodeAnnouncement>) -> usize {
        let mut count = 0;
        for ann in announcements {
            if ann.node_id != self.local_node {
                self.process_announcement(ann);
                count += 1;
            }
        }
        count
    }

    /// Mark stale peers as dead based on the TTL.
    pub fn sweep_stale(&self) -> usize {
        let now = Utc::now();
        let ttl = chrono::Duration::milliseconds(self.ttl_ms as i64);
        let mut peers = self.peers.lock().unwrap();
        let mut swept = 0;
        for peer in peers.values_mut() {
            if peer.alive && (now - peer.last_seen) >= ttl {
                peer.alive = false;
                swept += 1;
            }
        }
        swept
    }

    /// Return the count of alive peers.
    pub fn alive_count(&self) -> usize {
        self.peers
            .lock()
            .unwrap()
            .values()
            .filter(|p| p.alive)
            .count()
    }

    /// Return the local node ID.
    pub fn local_node(&self) -> NodeId {
        self.local_node
    }
}

/// When the `ruv-swarm` feature is enabled, provides topology-aware discovery
/// using `ruv-swarm-core`'s mesh and star topologies.
#[cfg(feature = "ruv-swarm")]
pub mod topology_discovery {
    use super::*;
    use ruv_swarm_core::topology::{Topology, TopologyType};

    /// Build a mesh topology from discovered peers.
    pub fn build_mesh_topology(peers: &[NodeAnnouncement]) -> Topology {
        let agent_ids: Vec<String> = peers.iter().map(|p| p.node_id.0.to_string()).collect();
        let refs: Vec<&str> = agent_ids.iter().map(|s| s.as_str()).collect();
        // ruv-swarm-core expects &[AgentId] which is Vec<String>
        Topology::mesh(&refs.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    /// Build a star topology with a designated coordinator.
    pub fn build_star_topology(
        coordinator: &NodeAnnouncement,
        peers: &[NodeAnnouncement],
    ) -> Topology {
        let center = coordinator.node_id.0.to_string();
        let agents: Vec<String> = peers.iter().map(|p| p.node_id.0.to_string()).collect();
        Topology::star(center, &agents)
    }

    /// Return the topology type best suited for a zone size.
    pub fn recommended_topology(peer_count: usize) -> TopologyType {
        if peer_count <= 5 {
            TopologyType::Mesh
        } else if peer_count <= 20 {
            TopologyType::Star
        } else {
            TopologyType::Clustered
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_announcement(zone: &str) -> NodeAnnouncement {
        NodeAnnouncement {
            node_id: NodeId::new(),
            zone_id: ZoneId::new(zone),
            address: "127.0.0.1:3000".to_string(),
            capabilities: vec!["inference".to_string()],
            announced_at: Utc::now(),
        }
    }

    #[test]
    fn test_announce_produces_record() {
        let node = NodeId::new();
        let disc = SwarmDiscovery::new(node, ZoneId::new("A"));
        let ann = disc.announce("10.0.0.1:3000".into(), vec!["gpu".into()]);
        assert_eq!(ann.node_id, node);
        assert_eq!(ann.zone_id.0, "A");
        assert_eq!(ann.capabilities, vec!["gpu"]);
    }

    #[test]
    fn test_discover_peers_after_join() {
        let local = NodeId::new();
        let disc = SwarmDiscovery::new(local, ZoneId::new("A"));

        let anns = vec![
            make_announcement("A"),
            make_announcement("B"),
            make_announcement("A"),
        ];
        let joined = disc.join(anns);
        assert_eq!(joined, 3);
        assert_eq!(disc.alive_count(), 3);
        assert_eq!(disc.discover_peers().len(), 3);
    }

    #[test]
    fn test_self_announcement_ignored() {
        let local = NodeId::new();
        let disc = SwarmDiscovery::new(local, ZoneId::new("A"));

        let self_ann = NodeAnnouncement {
            node_id: local,
            zone_id: ZoneId::new("A"),
            address: "self".into(),
            capabilities: vec![],
            announced_at: Utc::now(),
        };
        disc.process_announcement(self_ann);
        assert_eq!(disc.alive_count(), 0);
    }

    #[test]
    fn test_discover_peers_in_zone() {
        let local = NodeId::new();
        let disc = SwarmDiscovery::new(local, ZoneId::new("A"));

        disc.join(vec![
            make_announcement("A"),
            make_announcement("B"),
            make_announcement("A"),
        ]);

        let zone_a = disc.discover_peers_in_zone(&ZoneId::new("A"));
        assert_eq!(zone_a.len(), 2);

        let zone_b = disc.discover_peers_in_zone(&ZoneId::new("B"));
        assert_eq!(zone_b.len(), 1);
    }

    #[test]
    fn test_sweep_stale_peers() {
        let local = NodeId::new();
        let disc = SwarmDiscovery::new(local, ZoneId::new("A")).with_ttl_ms(0);

        disc.join(vec![make_announcement("A"), make_announcement("B")]);
        // With TTL=0 all peers are immediately stale.
        let swept = disc.sweep_stale();
        assert_eq!(swept, 2);
        assert_eq!(disc.alive_count(), 0);
        assert!(disc.discover_peers().is_empty());
    }

    // -----------------------------------------------------------------------
    // Topology discovery tests (feature = "ruv-swarm")
    // -----------------------------------------------------------------------

    #[cfg(feature = "ruv-swarm")]
    mod topology_tests {
        use super::*;
        use crate::discovery::topology_discovery::{build_mesh_topology, recommended_topology};
        use ruv_swarm_core::topology::TopologyType;

        #[test]
        fn test_build_mesh_topology() {
            let peers = vec![
                make_announcement("A"),
                make_announcement("A"),
                make_announcement("A"),
            ];
            let topo = build_mesh_topology(&peers);
            assert_eq!(topo.topology_type, TopologyType::Mesh);
        }

        #[test]
        fn test_recommended_topology_mesh_for_small() {
            assert_eq!(recommended_topology(3), TopologyType::Mesh);
        }

        #[test]
        fn test_recommended_topology_star_for_medium() {
            assert_eq!(recommended_topology(10), TopologyType::Star);
        }

        #[test]
        fn test_recommended_topology_clustered_for_large() {
            assert_eq!(recommended_topology(50), TopologyType::Clustered);
        }
    }
}
