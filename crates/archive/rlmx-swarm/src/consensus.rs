use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::types::{ConsensusType, MetricsUpdate, NodeId, SwarmError};

/// Trait for consensus protocol implementations.
#[async_trait]
pub trait ConsensusLayer: Send + Sync {
    /// Propose a value for consensus. Returns true if accepted.
    async fn propose(&self, value: Vec<u8>) -> Result<bool, SwarmError>;

    /// Get the current leader, if any.
    async fn current_leader(&self) -> Option<NodeId>;

    /// The consensus type this layer implements.
    fn consensus_type(&self) -> ConsensusType;
}

// ---------------------------------------------------------------------------
// PBFT Layer
// ---------------------------------------------------------------------------

/// An entry in the PBFT consensus log, tracking prepare/commit votes.
#[derive(Debug, Clone)]
pub struct PbftEntry {
    pub sequence: u64,
    pub operation: String,
    pub witness_id: Uuid,
    pub digest: [u8; 32],
    pub prepares: HashSet<NodeId>,
    pub commits: HashSet<NodeId>,
}

impl PbftEntry {
    /// Create a new PBFT log entry, computing the SHA-256 digest of the operation.
    pub fn new(sequence: u64, operation: String, witness_id: Uuid) -> Self {
        let digest = {
            let mut hasher = Sha256::new();
            hasher.update(operation.as_bytes());
            let result = hasher.finalize();
            let mut d = [0u8; 32];
            d.copy_from_slice(&result);
            d
        };
        Self {
            sequence,
            operation,
            witness_id,
            digest,
            prepares: HashSet::new(),
            commits: HashSet::new(),
        }
    }

    /// Verify that the stored digest matches the operation content.
    pub fn verify_digest(&self) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(self.operation.as_bytes());
        let result = hasher.finalize();
        self.digest[..] == result[..]
    }
}

/// Practical Byzantine Fault Tolerance — for critical state consensus.
/// Requires 3f+1 nodes and validates 2/3 agreement.
pub struct PbftLayer {
    pub replicas: Vec<NodeId>,
    pub leader: Option<NodeId>,
    pub view: u64,
    pub log: Vec<PbftEntry>,
    pub checkpoint_interval: u64,
}

impl PbftLayer {
    pub fn new(replicas: Vec<NodeId>) -> Self {
        let leader = replicas.first().copied();
        Self {
            replicas,
            leader,
            view: 0,
            log: Vec::new(),
            checkpoint_interval: 100,
        }
    }

    /// Maximum number of faulty nodes tolerable.
    fn max_faults(&self) -> usize {
        if self.replicas.len() < 4 {
            0
        } else {
            (self.replicas.len() - 1) / 3
        }
    }

    /// Minimum nodes needed for agreement (2f+1 out of 3f+1).
    fn quorum_size(&self) -> usize {
        let f = self.max_faults();
        2 * f + 1
    }
}

#[async_trait]
impl ConsensusLayer for PbftLayer {
    async fn propose(&self, _value: Vec<u8>) -> Result<bool, SwarmError> {
        // Simulated: check we have enough replicas for PBFT.
        let n = self.replicas.len();
        if n < 4 {
            return Err(SwarmError::ConsensusFailed(format!(
                "PBFT requires at least 4 nodes, have {n}"
            )));
        }
        // Simulate 2/3 agreement (all honest in simulation).
        let agreeing = n; // all nodes agree in simulation
        Ok(agreeing >= self.quorum_size())
    }

    async fn current_leader(&self) -> Option<NodeId> {
        self.leader
    }

    fn consensus_type(&self) -> ConsensusType {
        ConsensusType::Pbft
    }
}

// ---------------------------------------------------------------------------
// Raft Layer
// ---------------------------------------------------------------------------

/// Entry types stored in the Raft log for metadata consensus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RaftEntry {
    ClusterMembership(String),
    PlacementPolicyUpdate(String),
    TokenRevocation(Uuid),
    ConfigChange(String),
    ZoneReassignment {
        node: NodeId,
        from_zone: String,
        to_zone: String,
    },
}

/// Raft consensus — for metadata coordination.
pub struct RaftLayer {
    pub voters: Vec<NodeId>,
    pub leader: Option<NodeId>,
    pub term: u64,
    pub voted_for: Option<NodeId>,
    pub log: Vec<RaftEntry>,
    pub commit_index: u64,
    pub election_timeout_ms: u64,
    pub heartbeat_interval_ms: u64,
}

impl RaftLayer {
    pub fn new(voters: Vec<NodeId>) -> Self {
        let leader = voters.first().copied();
        Self {
            voters,
            leader,
            term: 0,
            voted_for: leader,
            log: Vec::new(),
            commit_index: 0,
            election_timeout_ms: 200,
            heartbeat_interval_ms: 50,
        }
    }

    /// Majority quorum size.
    fn majority(&self) -> usize {
        self.voters.len() / 2 + 1
    }
}

#[async_trait]
impl ConsensusLayer for RaftLayer {
    async fn propose(&self, _value: Vec<u8>) -> Result<bool, SwarmError> {
        if self.leader.is_none() {
            return Err(SwarmError::ConsensusFailed("no leader elected".into()));
        }
        // Simulate: leader replicates to majority.
        Ok(self.voters.len() >= self.majority())
    }

    async fn current_leader(&self) -> Option<NodeId> {
        self.leader
    }

    fn consensus_type(&self) -> ConsensusType {
        ConsensusType::Raft
    }
}

// ---------------------------------------------------------------------------
// Gossip Layer
// ---------------------------------------------------------------------------

/// State tracked per node in the gossip protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipState {
    pub generation: u64,
    pub data: HashMap<String, serde_json::Value>,
    pub last_update: DateTime<Utc>,
}

/// Gossip protocol — for health metrics and state dissemination.
pub struct GossipLayer {
    pub members: HashMap<NodeId, GossipState>,
    pub fanout: usize,
    pub interval_ms: u64,
    pub suspicion_timeout_ms: u64,
    pub dead_timeout_ms: u64,
    pub metrics_buffer: Vec<MetricsUpdate>,
}

impl GossipLayer {
    pub fn new(node_ids: Vec<NodeId>, fanout: usize) -> Self {
        let members = node_ids
            .into_iter()
            .map(|id| {
                (
                    id,
                    GossipState {
                        generation: 0,
                        data: HashMap::new(),
                        last_update: Utc::now(),
                    },
                )
            })
            .collect();
        Self {
            members,
            fanout,
            interval_ms: 500,
            suspicion_timeout_ms: 3000,
            dead_timeout_ms: 10000,
            metrics_buffer: Vec::new(),
        }
    }

    /// Update the gossip state for a node.
    pub fn update_state(&mut self, node_id: &NodeId, key: String, value: serde_json::Value) {
        if let Some(state) = self.members.get_mut(node_id) {
            state.generation += 1;
            state.data.insert(key, value);
            state.last_update = Utc::now();
        }
    }

    /// Get gossip state for a node.
    pub fn get_state(&self, node_id: &NodeId) -> Option<&GossipState> {
        self.members.get(node_id)
    }

    /// Push a metrics update into the gossip buffer.
    pub fn push_metrics(&mut self, update: MetricsUpdate) {
        self.metrics_buffer.push(update);
    }
}

#[async_trait]
impl ConsensusLayer for GossipLayer {
    async fn propose(&self, _value: Vec<u8>) -> Result<bool, SwarmError> {
        // Gossip is eventually consistent — always "succeeds" for dissemination.
        Ok(!self.members.is_empty())
    }

    async fn current_leader(&self) -> Option<NodeId> {
        // Gossip has no leader concept.
        None
    }

    fn consensus_type(&self) -> ConsensusType {
        ConsensusType::Gossip
    }
}

// ---------------------------------------------------------------------------
// Consensus Manager
// ---------------------------------------------------------------------------

/// Routes consensus requests to the appropriate protocol layer.
pub struct ConsensusManager {
    pub layers: HashMap<ConsensusType, Box<dyn ConsensusLayer + Send + Sync>>,
}

impl ConsensusManager {
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
        }
    }

    pub fn register(&mut self, layer: Box<dyn ConsensusLayer + Send + Sync>) {
        self.layers.insert(layer.consensus_type(), layer);
    }

    pub async fn propose(
        &self,
        consensus_type: ConsensusType,
        value: Vec<u8>,
    ) -> Result<bool, SwarmError> {
        let layer = self.layers.get(&consensus_type).ok_or_else(|| {
            SwarmError::ConsensusFailed(format!("no layer for {:?}", consensus_type))
        })?;
        layer.propose(value).await
    }

    /// Route a syscall type to the appropriate consensus layer per ADR-002.
    pub fn submit_for_syscall(&self, syscall_type: &str) -> ConsensusType {
        match syscall_type {
            "StateMutate" => ConsensusType::Pbft,
            "ClusterMembership" | "ConfigChange" | "TokenRevocation" => ConsensusType::Raft,
            _ => ConsensusType::Gossip,
        }
    }

    pub async fn leader(&self, consensus_type: ConsensusType) -> Option<NodeId> {
        if let Some(layer) = self.layers.get(&consensus_type) {
            layer.current_leader().await
        } else {
            None
        }
    }
}

impl Default for ConsensusManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_nodes(n: usize) -> Vec<NodeId> {
        (0..n).map(|_| NodeId::new()).collect()
    }

    // -----------------------------------------------------------------------
    // PBFT tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_pbft_requires_4_nodes() {
        let pbft = PbftLayer::new(make_nodes(3));
        let result = pbft.propose(b"value".to_vec()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pbft_4_nodes_succeeds() {
        let pbft = PbftLayer::new(make_nodes(4));
        let result = pbft.propose(b"value".to_vec()).await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_pbft_leader() {
        let replicas = make_nodes(4);
        let expected = replicas[0];
        let pbft = PbftLayer::new(replicas);
        assert_eq!(pbft.current_leader().await, Some(expected));
    }

    #[tokio::test]
    async fn test_pbft_defaults() {
        let pbft = PbftLayer::new(make_nodes(5));
        assert_eq!(pbft.view, 0);
        assert_eq!(pbft.checkpoint_interval, 100);
        assert!(pbft.log.is_empty());
    }

    #[test]
    fn test_pbft_entry_creation_and_digest() {
        let witness_id = Uuid::new_v4();
        let entry = PbftEntry::new(1, "StateMutate { key: v }".to_string(), witness_id);

        assert_eq!(entry.sequence, 1);
        assert_eq!(entry.witness_id, witness_id);
        assert!(entry.prepares.is_empty());
        assert!(entry.commits.is_empty());
        assert!(
            entry.verify_digest(),
            "digest should verify for unmodified entry"
        );
    }

    #[test]
    fn test_pbft_entry_digest_detects_tampering() {
        let mut entry = PbftEntry::new(1, "StateMutate { key: v }".to_string(), Uuid::new_v4());
        // Tamper with the operation after creation.
        entry.operation = "StateMutate { key: TAMPERED }".to_string();
        assert!(!entry.verify_digest(), "digest should fail after tampering");
    }

    #[test]
    fn test_pbft_entry_prepare_and_commit_tracking() {
        let nodes = make_nodes(4);
        let mut entry = PbftEntry::new(1, "StateMutate".to_string(), Uuid::new_v4());
        entry.prepares.insert(nodes[0]);
        entry.prepares.insert(nodes[1]);
        entry.prepares.insert(nodes[2]);
        assert_eq!(entry.prepares.len(), 3);

        entry.commits.insert(nodes[0]);
        entry.commits.insert(nodes[1]);
        assert_eq!(entry.commits.len(), 2);
    }

    // -----------------------------------------------------------------------
    // Raft tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_raft_propose_with_leader() {
        let raft = RaftLayer::new(make_nodes(3));
        assert!(raft.propose(b"val".to_vec()).await.unwrap());
    }

    #[tokio::test]
    async fn test_raft_no_leader() {
        let mut raft = RaftLayer::new(make_nodes(3));
        raft.leader = None;
        let result = raft.propose(b"val".to_vec()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_raft_leader_election() {
        let voters = make_nodes(5);
        let expected = voters[0];
        let raft = RaftLayer::new(voters);
        assert_eq!(raft.current_leader().await, Some(expected));
    }

    #[tokio::test]
    async fn test_raft_defaults() {
        let raft = RaftLayer::new(make_nodes(5));
        assert_eq!(raft.term, 0);
        assert_eq!(raft.commit_index, 0);
        assert_eq!(raft.election_timeout_ms, 200);
        assert_eq!(raft.heartbeat_interval_ms, 50);
        assert!(raft.log.is_empty());
    }

    #[test]
    fn test_raft_entry_serialization() {
        let entries = vec![
            RaftEntry::ClusterMembership("add node-7".to_string()),
            RaftEntry::PlacementPolicyUpdate("inference-optimized".to_string()),
            RaftEntry::TokenRevocation(Uuid::new_v4()),
            RaftEntry::ConfigChange("max_agents=10".to_string()),
            RaftEntry::ZoneReassignment {
                node: NodeId::new(),
                from_zone: "B".to_string(),
                to_zone: "A".to_string(),
            },
        ];
        for entry in &entries {
            let json = serde_json::to_string(entry).unwrap();
            let roundtrip: RaftEntry = serde_json::from_str(&json).unwrap();
            // Verify variant tag survived roundtrip.
            let json2 = serde_json::to_string(&roundtrip).unwrap();
            assert_eq!(json, json2);
        }
    }

    // -----------------------------------------------------------------------
    // Gossip tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_gossip_state_propagation() {
        let nodes = make_nodes(3);
        let n0 = nodes[0];
        let mut gossip = GossipLayer::new(nodes, 2);

        gossip.update_state(&n0, "cpu".into(), serde_json::json!(0.5));
        let state = gossip.get_state(&n0).unwrap();
        assert_eq!(state.generation, 1);
        assert_eq!(state.data["cpu"], serde_json::json!(0.5));
    }

    #[tokio::test]
    async fn test_gossip_propose_succeeds() {
        let gossip = GossipLayer::new(make_nodes(3), 2);
        assert!(gossip.propose(b"data".to_vec()).await.unwrap());
    }

    #[tokio::test]
    async fn test_gossip_no_leader() {
        let gossip = GossipLayer::new(make_nodes(3), 2);
        assert!(gossip.current_leader().await.is_none());
    }

    #[tokio::test]
    async fn test_gossip_defaults() {
        let gossip = GossipLayer::new(make_nodes(3), 3);
        assert_eq!(gossip.interval_ms, 500);
        assert_eq!(gossip.suspicion_timeout_ms, 3000);
        assert_eq!(gossip.dead_timeout_ms, 10000);
        assert!(gossip.metrics_buffer.is_empty());
    }

    #[test]
    fn test_gossip_metrics_buffer() {
        let nodes = make_nodes(3);
        let n0 = nodes[0];
        let mut gossip = GossipLayer::new(nodes, 2);

        let update = MetricsUpdate {
            node: n0,
            timestamp: Utc::now(),
            load: 0.8,
            memory_pressure: 0.4,
            inference_latency_p99_ms: 55,
            active_processes: 3,
            vector_segments: 7,
        };
        gossip.push_metrics(update);

        assert_eq!(gossip.metrics_buffer.len(), 1);
        assert_eq!(gossip.metrics_buffer[0].node, n0);
        assert!((gossip.metrics_buffer[0].load - 0.8).abs() < f32::EPSILON);
        assert_eq!(gossip.metrics_buffer[0].vector_segments, 7);
    }

    // -----------------------------------------------------------------------
    // ConsensusManager tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_consensus_manager_routing() {
        let mut mgr = ConsensusManager::new();
        mgr.register(Box::new(RaftLayer::new(make_nodes(3))));
        mgr.register(Box::new(GossipLayer::new(make_nodes(3), 2)));

        assert!(mgr
            .propose(ConsensusType::Raft, b"v".to_vec())
            .await
            .unwrap());
        assert!(mgr
            .propose(ConsensusType::Gossip, b"v".to_vec())
            .await
            .unwrap());
        assert!(mgr
            .propose(ConsensusType::Pbft, b"v".to_vec())
            .await
            .is_err());
    }

    #[test]
    fn test_submit_for_syscall_routing() {
        let mgr = ConsensusManager::new();

        assert_eq!(mgr.submit_for_syscall("StateMutate"), ConsensusType::Pbft);
        assert_eq!(
            mgr.submit_for_syscall("ClusterMembership"),
            ConsensusType::Raft
        );
        assert_eq!(mgr.submit_for_syscall("ConfigChange"), ConsensusType::Raft);
        assert_eq!(
            mgr.submit_for_syscall("TokenRevocation"),
            ConsensusType::Raft
        );
        assert_eq!(mgr.submit_for_syscall("VecSearch"), ConsensusType::Gossip);
        assert_eq!(mgr.submit_for_syscall("GraphQuery"), ConsensusType::Gossip);
        assert_eq!(mgr.submit_for_syscall("ProcessFork"), ConsensusType::Gossip);
    }
}
