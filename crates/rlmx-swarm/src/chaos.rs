use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::types::{NodeId, ZoneId};

/// Types of faults that can be injected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FaultType {
    /// Simulates a node crash.
    NodeCrash(NodeId),
    /// Simulates a network partition between two groups.
    NetworkPartition(Vec<NodeId>, Vec<NodeId>),
    /// Simulates a latency spike on a specific node.
    LatencySpike { node: NodeId, ms: u64 },
    /// Simulates a byzantine (malicious) node.
    Byzantine(NodeId),
    /// Simulates an entire zone going down.
    ZoneFailure(ZoneId),
    /// Simulates random message drops at a given rate (0.0–1.0).
    MessageDrop { rate: f64 },
}

/// Identifier for a tracked fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FaultId(pub Uuid);

impl FaultId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for FaultId {
    fn default() -> Self {
        Self::new()
    }
}

/// A currently active fault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveFault {
    pub id: FaultId,
    pub fault_type: FaultType,
    pub injected_at: DateTime<Utc>,
    pub duration: Option<Duration>,
}

/// Manages fault injection for chaos testing.
#[derive(Debug, Clone)]
pub struct FaultInjector {
    active_faults: Vec<ActiveFault>,
}

impl FaultInjector {
    pub fn new() -> Self {
        Self {
            active_faults: Vec::new(),
        }
    }

    /// Inject a fault and return its ID for later clearing.
    pub fn inject(&mut self, fault_type: FaultType) -> FaultId {
        let id = FaultId::new();
        self.active_faults.push(ActiveFault {
            id,
            fault_type,
            injected_at: Utc::now(),
            duration: None,
        });
        tracing::warn!(?id, "fault injected");
        id
    }

    /// Clear a specific fault by ID.
    pub fn clear(&mut self, fault_id: FaultId) {
        self.active_faults.retain(|f| f.id != fault_id);
        tracing::info!(?fault_id, "fault cleared");
    }

    /// Clear all active faults.
    pub fn clear_all(&mut self) {
        self.active_faults.clear();
        tracing::info!("all faults cleared");
    }

    /// Get a reference to all active faults.
    pub fn active_faults(&self) -> &[ActiveFault] {
        &self.active_faults
    }
}

impl Default for FaultInjector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_fault() {
        let mut injector = FaultInjector::new();
        let id = injector.inject(FaultType::NodeCrash(NodeId::new()));
        assert_eq!(injector.active_faults().len(), 1);
        assert_eq!(injector.active_faults()[0].id, id);
    }

    #[test]
    fn test_clear_fault() {
        let mut injector = FaultInjector::new();
        let id1 = injector.inject(FaultType::NodeCrash(NodeId::new()));
        let _id2 = injector.inject(FaultType::MessageDrop { rate: 0.5 });
        injector.clear(id1);
        assert_eq!(injector.active_faults().len(), 1);
    }

    #[test]
    fn test_clear_all() {
        let mut injector = FaultInjector::new();
        injector.inject(FaultType::NodeCrash(NodeId::new()));
        injector.inject(FaultType::Byzantine(NodeId::new()));
        injector.clear_all();
        assert!(injector.active_faults().is_empty());
    }

    #[test]
    fn test_network_partition() {
        let mut injector = FaultInjector::new();
        let a = vec![NodeId::new(), NodeId::new()];
        let b = vec![NodeId::new()];
        let id = injector.inject(FaultType::NetworkPartition(a, b));
        assert_eq!(injector.active_faults().len(), 1);
        injector.clear(id);
        assert!(injector.active_faults().is_empty());
    }

    #[test]
    fn test_latency_spike() {
        let mut injector = FaultInjector::new();
        let nid = NodeId::new();
        injector.inject(FaultType::LatencySpike { node: nid, ms: 500 });
        match &injector.active_faults()[0].fault_type {
            FaultType::LatencySpike { ms, .. } => assert_eq!(*ms, 500),
            _ => panic!("wrong fault type"),
        }
    }

    #[test]
    fn test_zone_failure() {
        let mut injector = FaultInjector::new();
        injector.inject(FaultType::ZoneFailure(ZoneId::new("zone-a")));
        assert_eq!(injector.active_faults().len(), 1);
    }
}
