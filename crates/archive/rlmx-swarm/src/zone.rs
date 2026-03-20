use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::node::SwarmNode;
use crate::types::{ConsensusType, NodeId, PlacementPolicy, SwarmError, ZoneId};

/// A zone within the swarm — a logical grouping of nodes sharing a consensus type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Zone {
    pub id: ZoneId,
    pub name: String,
    pub consensus_type: ConsensusType,
    pub nodes: HashMap<NodeId, SwarmNode>,
    pub policy: PlacementPolicy,
}

impl Zone {
    pub fn new(
        id: ZoneId,
        name: impl Into<String>,
        consensus_type: ConsensusType,
        policy: PlacementPolicy,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            consensus_type,
            nodes: HashMap::new(),
            policy,
        }
    }
}

/// Manages all zones in the swarm.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneManager {
    pub zones: HashMap<ZoneId, Zone>,
}

impl ZoneManager {
    pub fn new() -> Self {
        Self {
            zones: HashMap::new(),
        }
    }

    pub fn add_zone(&mut self, zone: Zone) {
        self.zones.insert(zone.id.clone(), zone);
    }

    pub fn add_node(&mut self, zone_id: &ZoneId, node: SwarmNode) -> Result<(), SwarmError> {
        let zone = self
            .zones
            .get_mut(zone_id)
            .ok_or_else(|| SwarmError::ZoneNotFound(zone_id.clone()))?;
        zone.nodes.insert(node.id, node);
        Ok(())
    }

    pub fn remove_node(&mut self, node_id: &NodeId) -> Result<SwarmNode, SwarmError> {
        for zone in self.zones.values_mut() {
            if let Some(node) = zone.nodes.remove(node_id) {
                return Ok(node);
            }
        }
        Err(SwarmError::NodeNotFound(*node_id))
    }

    pub fn get_zone(&self, zone_id: &ZoneId) -> Option<&Zone> {
        self.zones.get(zone_id)
    }

    pub fn get_node(&self, node_id: &NodeId) -> Option<&SwarmNode> {
        for zone in self.zones.values() {
            if let Some(node) = zone.nodes.get(node_id) {
                return Some(node);
            }
        }
        None
    }

    pub fn healthy_nodes(&self, zone_id: &ZoneId) -> Vec<&SwarmNode> {
        self.zones
            .get(zone_id)
            .map(|zone| zone.nodes.values().filter(|n| n.is_healthy()).collect())
            .unwrap_or_default()
    }

    pub fn total_nodes(&self) -> usize {
        self.zones.values().map(|z| z.nodes.len()).sum()
    }

    pub fn find_zone_for_policy(&self, policy: PlacementPolicy) -> Option<&Zone> {
        self.zones.values().find(|z| z.policy == policy)
    }
}

impl Default for ZoneManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::NodeProfile;

    fn make_zone(id: &str, name: &str, policy: PlacementPolicy) -> Zone {
        Zone::new(ZoneId::new(id), name, ConsensusType::Raft, policy)
    }

    fn make_node(zone_id: &str) -> SwarmNode {
        SwarmNode::new(
            ZoneId::new(zone_id),
            NodeProfile {
                cpu_cores: 4,
                memory_mb: 8192,
                has_gpu: false,
                gpu_type: None,
                architecture: "x86_64".into(),
            },
            "127.0.0.1:9000",
        )
    }

    #[test]
    fn test_add_and_get_zone() {
        let mut mgr = ZoneManager::new();
        mgr.add_zone(make_zone("z1", "Zone A", PlacementPolicy::Any));
        assert!(mgr.get_zone(&ZoneId::new("z1")).is_some());
        assert!(mgr.get_zone(&ZoneId::new("z2")).is_none());
    }

    #[test]
    fn test_add_node_to_zone() {
        let mut mgr = ZoneManager::new();
        mgr.add_zone(make_zone("z1", "Zone A", PlacementPolicy::Any));
        let node = make_node("z1");
        let node_id = node.id;
        mgr.add_node(&ZoneId::new("z1"), node).unwrap();
        assert!(mgr.get_node(&node_id).is_some());
    }

    #[test]
    fn test_add_node_missing_zone() {
        let mut mgr = ZoneManager::new();
        let node = make_node("z1");
        let result = mgr.add_node(&ZoneId::new("z999"), node);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_node() {
        let mut mgr = ZoneManager::new();
        mgr.add_zone(make_zone("z1", "Zone A", PlacementPolicy::Any));
        let node = make_node("z1");
        let node_id = node.id;
        mgr.add_node(&ZoneId::new("z1"), node).unwrap();
        let removed = mgr.remove_node(&node_id).unwrap();
        assert_eq!(removed.id, node_id);
        assert!(mgr.get_node(&node_id).is_none());
    }

    #[test]
    fn test_remove_node_not_found() {
        let mut mgr = ZoneManager::new();
        mgr.add_zone(make_zone("z1", "Zone A", PlacementPolicy::Any));
        let result = mgr.remove_node(&crate::types::NodeId::new());
        assert!(result.is_err());
    }

    #[test]
    fn test_healthy_nodes() {
        let mut mgr = ZoneManager::new();
        mgr.add_zone(make_zone("z1", "Zone A", PlacementPolicy::Any));

        let mut n1 = make_node("z1");
        n1.update_heartbeat(); // becomes Online
        let mut n2 = make_node("z1");
        n2.status = crate::node::NodeStatus::Offline;

        mgr.add_node(&ZoneId::new("z1"), n1).unwrap();
        mgr.add_node(&ZoneId::new("z1"), n2).unwrap();

        let healthy = mgr.healthy_nodes(&ZoneId::new("z1"));
        assert_eq!(healthy.len(), 1);
    }

    #[test]
    fn test_total_nodes() {
        let mut mgr = ZoneManager::new();
        mgr.add_zone(make_zone("z1", "Zone A", PlacementPolicy::Any));
        mgr.add_zone(make_zone("z2", "Zone B", PlacementPolicy::ComputeHeavy));
        mgr.add_node(&ZoneId::new("z1"), make_node("z1")).unwrap();
        mgr.add_node(&ZoneId::new("z1"), make_node("z1")).unwrap();
        mgr.add_node(&ZoneId::new("z2"), make_node("z2")).unwrap();
        assert_eq!(mgr.total_nodes(), 3);
    }

    #[test]
    fn test_find_zone_for_policy() {
        let mut mgr = ZoneManager::new();
        mgr.add_zone(make_zone("z1", "Zone A", PlacementPolicy::ComputeHeavy));
        mgr.add_zone(make_zone("z2", "Zone B", PlacementPolicy::EdgeRelay));

        let found = mgr.find_zone_for_policy(PlacementPolicy::EdgeRelay);
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, ZoneId::new("z2"));

        assert!(mgr
            .find_zone_for_policy(PlacementPolicy::InferenceOptimized)
            .is_none());
    }
}
