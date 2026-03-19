use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::types::{NodeId, NodeProfile, PlacementPolicy, ZoneId};

/// Status of a swarm node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Online,
    Degraded,
    Offline,
    Joining,
}

/// A node participating in the swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmNode {
    pub id: NodeId,
    pub zone: ZoneId,
    pub profile: NodeProfile,
    pub capabilities: HashSet<String>,
    pub status: NodeStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub address: String,
}

impl SwarmNode {
    /// Create a new swarm node with the given profile.
    pub fn new(zone: ZoneId, profile: NodeProfile, address: impl Into<String>) -> Self {
        Self {
            id: NodeId::new(),
            zone,
            profile,
            capabilities: HashSet::new(),
            status: NodeStatus::Joining,
            last_heartbeat: Utc::now(),
            address: address.into(),
        }
    }

    /// Returns true if the node is online or degraded (still functional).
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, NodeStatus::Online | NodeStatus::Degraded)
    }

    /// Update the heartbeat timestamp to now and set status to Online.
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
        if self.status == NodeStatus::Joining {
            self.status = NodeStatus::Online;
        }
    }

    /// Check whether this node matches the given placement policy.
    pub fn matches_policy(&self, policy: PlacementPolicy) -> bool {
        match policy {
            PlacementPolicy::Any => true,
            PlacementPolicy::ComputeHeavy => self.profile.cpu_cores >= 8,
            PlacementPolicy::InferenceOptimized => self.profile.has_gpu,
            PlacementPolicy::EdgeRelay => {
                self.profile.cpu_cores <= 4 && self.profile.memory_mb <= 4096
            }
            // Browser workers are not physical nodes; this policy never matches a SwarmNode.
            PlacementPolicy::BrowserCompute => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_profile(cores: u32, mem: u64, gpu: bool) -> NodeProfile {
        NodeProfile {
            cpu_cores: cores,
            memory_mb: mem,
            has_gpu: gpu,
            gpu_type: if gpu { Some("cuda".into()) } else { None },
            architecture: "x86_64".into(),
        }
    }

    #[test]
    fn test_new_node_is_joining() {
        let node = SwarmNode::new(
            ZoneId::new("z1"),
            make_profile(4, 8192, false),
            "127.0.0.1:9000",
        );
        assert_eq!(node.status, NodeStatus::Joining);
        assert!(!node.is_healthy());
    }

    #[test]
    fn test_heartbeat_transitions_to_online() {
        let mut node = SwarmNode::new(
            ZoneId::new("z1"),
            make_profile(4, 8192, false),
            "127.0.0.1:9000",
        );
        node.update_heartbeat();
        assert_eq!(node.status, NodeStatus::Online);
        assert!(node.is_healthy());
    }

    #[test]
    fn test_degraded_is_healthy() {
        let mut node = SwarmNode::new(
            ZoneId::new("z1"),
            make_profile(4, 8192, false),
            "127.0.0.1:9000",
        );
        node.status = NodeStatus::Degraded;
        assert!(node.is_healthy());
    }

    #[test]
    fn test_offline_is_not_healthy() {
        let mut node = SwarmNode::new(
            ZoneId::new("z1"),
            make_profile(4, 8192, false),
            "127.0.0.1:9000",
        );
        node.status = NodeStatus::Offline;
        assert!(!node.is_healthy());
    }

    #[test]
    fn test_matches_policy_any() {
        let node = SwarmNode::new(
            ZoneId::new("z1"),
            make_profile(2, 2048, false),
            "127.0.0.1:9000",
        );
        assert!(node.matches_policy(PlacementPolicy::Any));
    }

    #[test]
    fn test_matches_policy_compute_heavy() {
        let node = SwarmNode::new(
            ZoneId::new("z1"),
            make_profile(16, 32768, false),
            "127.0.0.1:9000",
        );
        assert!(node.matches_policy(PlacementPolicy::ComputeHeavy));

        let small = SwarmNode::new(
            ZoneId::new("z1"),
            make_profile(4, 8192, false),
            "127.0.0.1:9001",
        );
        assert!(!small.matches_policy(PlacementPolicy::ComputeHeavy));
    }

    #[test]
    fn test_matches_policy_inference() {
        let gpu_node = SwarmNode::new(
            ZoneId::new("z1"),
            make_profile(8, 16384, true),
            "127.0.0.1:9000",
        );
        assert!(gpu_node.matches_policy(PlacementPolicy::InferenceOptimized));

        let cpu_node = SwarmNode::new(
            ZoneId::new("z1"),
            make_profile(8, 16384, false),
            "127.0.0.1:9001",
        );
        assert!(!cpu_node.matches_policy(PlacementPolicy::InferenceOptimized));
    }

    #[test]
    fn test_matches_policy_edge_relay() {
        let edge = SwarmNode::new(
            ZoneId::new("z1"),
            make_profile(4, 4096, false),
            "127.0.0.1:9000",
        );
        assert!(edge.matches_policy(PlacementPolicy::EdgeRelay));

        let big = SwarmNode::new(
            ZoneId::new("z1"),
            make_profile(8, 8192, false),
            "127.0.0.1:9001",
        );
        assert!(!big.matches_policy(PlacementPolicy::EdgeRelay));
    }
}
