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

    pub fn verify_digest(&self) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(self.operation.as_bytes());
        let result = hasher.finalize();
        self.digest[..] == result[..]
    }
}

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
    fn max_faults(&self) -> usize {
        if self.replicas.len() < 4 {
            0
        } else {
            (self.replicas.len() - 1) / 3
        }
    }
    fn quorum_size(&self) -> usize {
        let f = self.max_faults();
        2 * f + 1
    }
}

#[async_trait]
impl ConsensusLayer for PbftLayer {
    async fn propose(&self, _value: Vec<u8>) -> Result<bool, SwarmError> {
        let n = self.replicas.len();
        if n < 4 {
            return Err(SwarmError::ConsensusFailed(format!(
                "PBFT requires at least 4 nodes, have {n}"
            )));
        }
        Ok(n >= self.quorum_size())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GossipState {
    pub generation: u64,
    pub data: HashMap<String, serde_json::Value>,
    pub last_update: DateTime<Utc>,
}

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
    pub fn update_state(&mut self, node_id: &NodeId, key: String, value: serde_json::Value) {
        if let Some(state) = self.members.get_mut(node_id) {
            state.generation += 1;
            state.data.insert(key, value);
            state.last_update = Utc::now();
        }
    }
    pub fn get_state(&self, node_id: &NodeId) -> Option<&GossipState> {
        self.members.get(node_id)
    }
    pub fn push_metrics(&mut self, update: MetricsUpdate) {
        // Cap buffer to prevent unbounded growth.
        if self.metrics_buffer.len() >= 1000 {
            self.metrics_buffer.drain(..1);
        }
        self.metrics_buffer.push(update);
    }
}

#[async_trait]
impl ConsensusLayer for GossipLayer {
    async fn propose(&self, _value: Vec<u8>) -> Result<bool, SwarmError> {
        Ok(!self.members.is_empty())
    }
    async fn current_leader(&self) -> Option<NodeId> {
        None
    }
    fn consensus_type(&self) -> ConsensusType {
        ConsensusType::Gossip
    }
}

// ---------------------------------------------------------------------------
// Consensus Manager
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Feature-gated: QuDAG consensus (ADR-024)
// ---------------------------------------------------------------------------
#[cfg(feature = "qudag")]
pub mod qudag_integration {
    use super::*;
    use qudag_dag::{ConsensusConfig as QrConfig, DAGConsensus, Vertex, VertexId};
    use std::sync::Mutex;
    use std::time::Duration;

    #[derive(Debug, Clone)]
    pub struct QuDagConfig {
        pub query_sample_size: usize,
        pub finality_threshold: f64,
        pub finality_timeout_ms: u64,
        pub confirmation_depth: usize,
    }
    impl Default for QuDagConfig {
        fn default() -> Self {
            Self {
                query_sample_size: 10,
                finality_threshold: 0.8,
                finality_timeout_ms: 5000,
                confirmation_depth: 3,
            }
        }
    }

    pub struct QuDagConsensusLayer {
        dag: Mutex<DAGConsensus>,
        replicas: Vec<NodeId>,
        vertex_count: Mutex<u64>,
    }

    impl QuDagConsensusLayer {
        pub fn new(replicas: Vec<NodeId>) -> Self {
            Self::with_config(replicas, QuDagConfig::default())
        }
        pub fn with_config(replicas: Vec<NodeId>, config: QuDagConfig) -> Self {
            let qr_config = QrConfig {
                query_sample_size: config.query_sample_size,
                finality_threshold: config.finality_threshold,
                finality_timeout: Duration::from_millis(config.finality_timeout_ms),
                confirmation_depth: config.confirmation_depth,
            };
            Self {
                dag: Mutex::new(DAGConsensus::with_config(qr_config)),
                replicas,
                vertex_count: Mutex::new(0),
            }
        }
        pub fn vertex_count(&self) -> u64 {
            *self.vertex_count.lock().unwrap()
        }
        pub fn tips(&self) -> Vec<String> {
            self.dag.lock().unwrap().get_tips()
        }
        pub fn contains(&self, payload: &[u8]) -> bool {
            self.dag.lock().unwrap().contains_message(payload)
        }
        pub fn replica_count(&self) -> usize {
            self.replicas.len()
        }
    }

    #[async_trait]
    impl ConsensusLayer for QuDagConsensusLayer {
        async fn propose(&self, value: Vec<u8>) -> Result<bool, SwarmError> {
            let mut dag = self
                .dag
                .lock()
                .map_err(|e| SwarmError::ConsensusFailed(format!("DAG lock poisoned: {e}")))?;
            let vertex_id = VertexId::from_bytes(value.clone());
            let vertex = Vertex::new(vertex_id, value, std::collections::HashSet::new());
            match dag.add_vertex(vertex) {
                Ok(()) => {
                    *self.vertex_count.lock().unwrap() += 1;
                    Ok(true)
                }
                Err(e) => Err(SwarmError::ConsensusFailed(format!(
                    "QuDAG vertex rejected: {e}"
                ))),
            }
        }
        async fn current_leader(&self) -> Option<NodeId> {
            None
        }
        fn consensus_type(&self) -> ConsensusType {
            ConsensusType::QuDag
        }
    }

    pub struct QrSigner {
        keypair: qudag_crypto::MlDsaKeyPair,
    }
    impl QrSigner {
        pub fn generate() -> Result<Self, SwarmError> {
            let keypair = qudag_crypto::MlDsa::generate_keypair()
                .map_err(|e| SwarmError::ConsensusFailed(format!("QR keygen failed: {e}")))?;
            Ok(Self { keypair })
        }
        pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, SwarmError> {
            qudag_crypto::MlDsa::sign(data, &self.keypair)
                .map_err(|e| SwarmError::ConsensusFailed(format!("QR sign failed: {e}")))
        }
        pub fn verify(&self, data: &[u8], signature: &[u8]) -> Result<bool, SwarmError> {
            qudag_crypto::MlDsa::verify(data, signature, &self.keypair.public_key)
                .map_err(|e| SwarmError::ConsensusFailed(format!("QR verify failed: {e}")))
        }
        pub fn public_key(&self) -> &qudag_crypto::MlDsaPublicKey {
            &self.keypair.public_key
        }
    }
}

// ---------------------------------------------------------------------------
// Feature-gated: RuVector Raft consensus (ADR-024)
// ---------------------------------------------------------------------------
#[cfg(feature = "ruvector-consensus")]
pub mod raft_integration {
    use super::*;
    use ruvector_raft::{PersistentState, RaftNodeConfig, RaftState};
    use std::sync::Mutex;

    #[derive(Debug, Clone)]
    pub struct RuVectorRaftConfig {
        pub election_timeout_min_ms: u64,
        pub election_timeout_max_ms: u64,
        pub heartbeat_interval_ms: u64,
        pub max_entries_per_message: usize,
    }
    impl Default for RuVectorRaftConfig {
        fn default() -> Self {
            Self {
                election_timeout_min_ms: 150,
                election_timeout_max_ms: 300,
                heartbeat_interval_ms: 50,
                max_entries_per_message: 100,
            }
        }
    }

    struct RaftBridgeState {
        persistent: PersistentState,
        current_state: RaftState,
        leader_id: Option<NodeId>,
        proposals_accepted: u64,
    }

    pub struct RuVectorRaftConsensus {
        voters: Vec<NodeId>,
        state: Mutex<RaftBridgeState>,
    }

    impl RuVectorRaftConsensus {
        pub fn new(voters: Vec<NodeId>) -> Self {
            Self::with_config(voters, RuVectorRaftConfig::default())
        }
        pub fn with_config(voters: Vec<NodeId>, config: RuVectorRaftConfig) -> Self {
            let node_id_str = voters
                .first()
                .map(|n| n.0.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let cluster: Vec<String> = voters.iter().map(|v| v.0.to_string()).collect();
            let mut raft_cfg = RaftNodeConfig::new(node_id_str, cluster);
            raft_cfg.election_timeout_min = config.election_timeout_min_ms;
            raft_cfg.election_timeout_max = config.election_timeout_max_ms;
            raft_cfg.heartbeat_interval = config.heartbeat_interval_ms;
            raft_cfg.max_entries_per_message = config.max_entries_per_message;
            let leader = voters.first().copied();
            Self {
                voters,
                state: Mutex::new(RaftBridgeState {
                    persistent: PersistentState::new(),
                    current_state: RaftState::Follower,
                    leader_id: leader,
                    proposals_accepted: 0,
                }),
            }
        }
        pub fn current_term(&self) -> u64 {
            self.state.lock().unwrap().persistent.current_term
        }
        pub fn raft_state(&self) -> String {
            match self.state.lock().unwrap().current_state {
                RaftState::Follower => "Follower".to_string(),
                RaftState::Candidate => "Candidate".to_string(),
                RaftState::Leader => "Leader".to_string(),
            }
        }
        pub fn proposals_accepted(&self) -> u64 {
            self.state.lock().unwrap().proposals_accepted
        }
        pub fn voter_count(&self) -> usize {
            self.voters.len()
        }
        pub fn simulate_election(&self, new_leader: NodeId) -> Result<(), SwarmError> {
            if !self.voters.contains(&new_leader) {
                return Err(SwarmError::ConsensusFailed(
                    "proposed leader not in voter set".into(),
                ));
            }
            let mut st = self.state.lock().unwrap();
            st.persistent.increment_term();
            st.leader_id = Some(new_leader);
            st.current_state = RaftState::Leader;
            Ok(())
        }
    }

    #[async_trait]
    impl ConsensusLayer for RuVectorRaftConsensus {
        async fn propose(&self, _value: Vec<u8>) -> Result<bool, SwarmError> {
            let mut st = self
                .state
                .lock()
                .map_err(|e| SwarmError::ConsensusFailed(format!("lock poisoned: {e}")))?;
            if st.leader_id.is_none() {
                return Err(SwarmError::ConsensusFailed("no leader elected".into()));
            }
            let majority = self.voters.len() / 2 + 1;
            if self.voters.len() >= majority {
                st.proposals_accepted += 1;
                Ok(true)
            } else {
                Ok(false)
            }
        }
        async fn current_leader(&self) -> Option<NodeId> {
            self.state.lock().ok().and_then(|st| st.leader_id)
        }
        fn consensus_type(&self) -> ConsensusType {
            ConsensusType::Raft
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    fn make_nodes(n: usize) -> Vec<NodeId> {
        (0..n).map(|_| NodeId::new()).collect()
    }

    #[tokio::test]
    async fn test_pbft_requires_4_nodes() {
        let pbft = PbftLayer::new(make_nodes(3));
        assert!(pbft.propose(b"value".to_vec()).await.is_err());
    }
    #[tokio::test]
    async fn test_pbft_4_nodes_succeeds() {
        let pbft = PbftLayer::new(make_nodes(4));
        assert!(pbft.propose(b"value".to_vec()).await.unwrap());
    }
    #[tokio::test]
    async fn test_pbft_leader() {
        let r = make_nodes(4);
        let e = r[0];
        let p = PbftLayer::new(r);
        assert_eq!(p.current_leader().await, Some(e));
    }
    #[tokio::test]
    async fn test_pbft_defaults() {
        let p = PbftLayer::new(make_nodes(5));
        assert_eq!(p.view, 0);
        assert_eq!(p.checkpoint_interval, 100);
        assert!(p.log.is_empty());
    }
    #[test]
    fn test_pbft_entry_creation_and_digest() {
        let w = Uuid::new_v4();
        let e = PbftEntry::new(1, "StateMutate { key: v }".to_string(), w);
        assert_eq!(e.sequence, 1);
        assert_eq!(e.witness_id, w);
        assert!(e.prepares.is_empty());
        assert!(e.commits.is_empty());
        assert!(
            e.verify_digest(),
            "digest should verify for unmodified entry"
        );
    }
    #[test]
    fn test_pbft_entry_digest_detects_tampering() {
        let mut e = PbftEntry::new(1, "StateMutate { key: v }".to_string(), Uuid::new_v4());
        e.operation = "StateMutate { key: TAMPERED }".to_string();
        assert!(!e.verify_digest(), "digest should fail after tampering");
    }
    #[test]
    fn test_pbft_entry_prepare_and_commit_tracking() {
        let n = make_nodes(4);
        let mut e = PbftEntry::new(1, "StateMutate".to_string(), Uuid::new_v4());
        e.prepares.insert(n[0]);
        e.prepares.insert(n[1]);
        e.prepares.insert(n[2]);
        assert_eq!(e.prepares.len(), 3);
        e.commits.insert(n[0]);
        e.commits.insert(n[1]);
        assert_eq!(e.commits.len(), 2);
    }
    #[tokio::test]
    async fn test_raft_propose_with_leader() {
        assert!(RaftLayer::new(make_nodes(3))
            .propose(b"val".to_vec())
            .await
            .unwrap());
    }
    #[tokio::test]
    async fn test_raft_no_leader() {
        let mut r = RaftLayer::new(make_nodes(3));
        r.leader = None;
        assert!(r.propose(b"val".to_vec()).await.is_err());
    }
    #[tokio::test]
    async fn test_raft_leader_election() {
        let v = make_nodes(5);
        let e = v[0];
        let r = RaftLayer::new(v);
        assert_eq!(r.current_leader().await, Some(e));
    }
    #[tokio::test]
    async fn test_raft_defaults() {
        let r = RaftLayer::new(make_nodes(5));
        assert_eq!(r.term, 0);
        assert_eq!(r.commit_index, 0);
        assert_eq!(r.election_timeout_ms, 200);
        assert_eq!(r.heartbeat_interval_ms, 50);
        assert!(r.log.is_empty());
    }
    #[test]
    fn test_raft_entry_serialization() {
        let entries = vec![
            RaftEntry::ClusterMembership("add node-7".into()),
            RaftEntry::PlacementPolicyUpdate("inference-optimized".into()),
            RaftEntry::TokenRevocation(Uuid::new_v4()),
            RaftEntry::ConfigChange("max_agents=10".into()),
            RaftEntry::ZoneReassignment {
                node: NodeId::new(),
                from_zone: "B".into(),
                to_zone: "A".into(),
            },
        ];
        for e in &entries {
            let j = serde_json::to_string(e).unwrap();
            let r: RaftEntry = serde_json::from_str(&j).unwrap();
            assert_eq!(j, serde_json::to_string(&r).unwrap());
        }
    }
    #[tokio::test]
    async fn test_gossip_state_propagation() {
        let n = make_nodes(3);
        let n0 = n[0];
        let mut g = GossipLayer::new(n, 2);
        g.update_state(&n0, "cpu".into(), serde_json::json!(0.5));
        let s = g.get_state(&n0).unwrap();
        assert_eq!(s.generation, 1);
        assert_eq!(s.data["cpu"], serde_json::json!(0.5));
    }
    #[tokio::test]
    async fn test_gossip_propose_succeeds() {
        assert!(GossipLayer::new(make_nodes(3), 2)
            .propose(b"data".to_vec())
            .await
            .unwrap());
    }
    #[tokio::test]
    async fn test_gossip_no_leader() {
        assert!(GossipLayer::new(make_nodes(3), 2)
            .current_leader()
            .await
            .is_none());
    }
    #[tokio::test]
    async fn test_gossip_defaults() {
        let g = GossipLayer::new(make_nodes(3), 3);
        assert_eq!(g.interval_ms, 500);
        assert_eq!(g.suspicion_timeout_ms, 3000);
        assert_eq!(g.dead_timeout_ms, 10000);
        assert!(g.metrics_buffer.is_empty());
    }
    #[test]
    fn test_gossip_metrics_buffer() {
        let n = make_nodes(3);
        let n0 = n[0];
        let mut g = GossipLayer::new(n, 2);
        g.push_metrics(MetricsUpdate {
            node: n0,
            timestamp: Utc::now(),
            load: 0.8,
            memory_pressure: 0.4,
            inference_latency_p99_ms: 55,
            active_processes: 3,
            vector_segments: 7,
        });
        assert_eq!(g.metrics_buffer.len(), 1);
        assert_eq!(g.metrics_buffer[0].node, n0);
        assert!((g.metrics_buffer[0].load - 0.8).abs() < f32::EPSILON);
        assert_eq!(g.metrics_buffer[0].vector_segments, 7);
    }
    #[tokio::test]
    async fn test_consensus_manager_routing() {
        let mut m = ConsensusManager::new();
        m.register(Box::new(RaftLayer::new(make_nodes(3))));
        m.register(Box::new(GossipLayer::new(make_nodes(3), 2)));
        assert!(m.propose(ConsensusType::Raft, b"v".to_vec()).await.unwrap());
        assert!(m
            .propose(ConsensusType::Gossip, b"v".to_vec())
            .await
            .unwrap());
        assert!(m.propose(ConsensusType::Pbft, b"v".to_vec()).await.is_err());
    }
    #[test]
    fn test_submit_for_syscall_routing() {
        let m = ConsensusManager::new();
        assert_eq!(m.submit_for_syscall("StateMutate"), ConsensusType::Pbft);
        assert_eq!(
            m.submit_for_syscall("ClusterMembership"),
            ConsensusType::Raft
        );
        assert_eq!(m.submit_for_syscall("ConfigChange"), ConsensusType::Raft);
        assert_eq!(m.submit_for_syscall("TokenRevocation"), ConsensusType::Raft);
        assert_eq!(m.submit_for_syscall("VecSearch"), ConsensusType::Gossip);
        assert_eq!(m.submit_for_syscall("GraphQuery"), ConsensusType::Gossip);
        assert_eq!(m.submit_for_syscall("ProcessFork"), ConsensusType::Gossip);
    }

    #[cfg(feature = "qudag")]
    mod qudag_tests {
        use super::*;
        use crate::consensus::qudag_integration::{QrSigner, QuDagConfig, QuDagConsensusLayer};
        #[tokio::test]
        async fn test_qudag_propose_accepted() {
            let l = QuDagConsensusLayer::new(make_nodes(4));
            assert!(l.propose(b"hello-dag".to_vec()).await.unwrap());
            assert_eq!(l.vertex_count(), 1);
        }
        #[tokio::test]
        async fn test_qudag_duplicate_rejected() {
            let l = QuDagConsensusLayer::new(make_nodes(4));
            l.propose(b"dup".to_vec()).await.unwrap();
            assert!(l.propose(b"dup".to_vec()).await.is_err());
        }
        #[tokio::test]
        async fn test_qudag_leaderless() {
            assert!(QuDagConsensusLayer::new(make_nodes(3))
                .current_leader()
                .await
                .is_none());
        }
        #[tokio::test]
        async fn test_qudag_consensus_type_is_qudag() {
            assert_eq!(
                QuDagConsensusLayer::new(make_nodes(3)).consensus_type(),
                ConsensusType::QuDag
            );
        }
        #[tokio::test]
        async fn test_qudag_tips_grow() {
            let l = QuDagConsensusLayer::new(make_nodes(3));
            assert!(l.tips().is_empty());
            l.propose(b"a".to_vec()).await.unwrap();
            assert_eq!(l.tips().len(), 1);
            l.propose(b"b".to_vec()).await.unwrap();
            assert_eq!(l.tips().len(), 2);
        }
        #[tokio::test]
        async fn test_qudag_contains() {
            let l = QuDagConsensusLayer::new(make_nodes(3));
            l.propose(b"find".to_vec()).await.unwrap();
            assert!(l.contains(b"find"));
            assert!(!l.contains(b"nope"));
        }
        #[test]
        fn test_qudag_custom_config() {
            let c = QuDagConfig {
                query_sample_size: 20,
                finality_threshold: 0.9,
                finality_timeout_ms: 10000,
                confirmation_depth: 5,
            };
            let l = QuDagConsensusLayer::with_config(make_nodes(5), c);
            assert_eq!(l.replica_count(), 5);
            assert_eq!(l.vertex_count(), 0);
        }
        #[test]
        fn test_qr_signer_sign_verify() {
            let s = QrSigner::generate().unwrap();
            let sig = s.sign(b"data").unwrap();
            assert!(s.verify(b"data", &sig).unwrap());
        }
        #[tokio::test]
        async fn test_qudag_in_manager() {
            let mut m = ConsensusManager::new();
            m.register(Box::new(QuDagConsensusLayer::new(make_nodes(4))));
            assert!(m.propose(ConsensusType::QuDag, b"v".to_vec()).await.unwrap());
        }
    }

    #[cfg(feature = "ruvector-consensus")]
    mod ruvector_raft_tests {
        use super::*;
        use crate::consensus::raft_integration::{RuVectorRaftConfig, RuVectorRaftConsensus};
        #[tokio::test]
        async fn test_ruvector_raft_propose() {
            let l = RuVectorRaftConsensus::new(make_nodes(3));
            assert!(l.propose(b"v".to_vec()).await.unwrap());
            assert_eq!(l.proposals_accepted(), 1);
        }
        #[tokio::test]
        async fn test_ruvector_raft_multi_propose() {
            let l = RuVectorRaftConsensus::new(make_nodes(3));
            l.propose(b"1".to_vec()).await.unwrap();
            l.propose(b"2".to_vec()).await.unwrap();
            l.propose(b"3".to_vec()).await.unwrap();
            assert_eq!(l.proposals_accepted(), 3);
        }
        #[tokio::test]
        async fn test_ruvector_raft_leader() {
            let v = make_nodes(5);
            let e = v[0];
            let l = RuVectorRaftConsensus::new(v);
            assert_eq!(l.current_leader().await, Some(e));
        }
        #[tokio::test]
        async fn test_ruvector_raft_type() {
            assert_eq!(
                RuVectorRaftConsensus::new(make_nodes(3)).consensus_type(),
                ConsensusType::Raft
            );
        }
        #[test]
        fn test_ruvector_raft_initial() {
            let l = RuVectorRaftConsensus::new(make_nodes(3));
            assert_eq!(l.current_term(), 0);
            assert_eq!(l.raft_state(), "Follower");
        }
        #[test]
        fn test_ruvector_raft_election() {
            let v = make_nodes(5);
            let nl = v[2];
            let l = RuVectorRaftConsensus::new(v);
            l.simulate_election(nl).unwrap();
            assert_eq!(l.current_term(), 1);
            assert_eq!(l.raft_state(), "Leader");
        }
        #[test]
        fn test_ruvector_raft_invalid_election() {
            let l = RuVectorRaftConsensus::new(make_nodes(3));
            assert!(l.simulate_election(NodeId::new()).is_err());
        }
        #[test]
        fn test_ruvector_raft_config() {
            let c = RuVectorRaftConfig {
                election_timeout_min_ms: 500,
                election_timeout_max_ms: 1000,
                heartbeat_interval_ms: 100,
                max_entries_per_message: 50,
            };
            assert_eq!(
                RuVectorRaftConsensus::with_config(make_nodes(5), c).voter_count(),
                5
            );
        }
        #[tokio::test]
        async fn test_ruvector_raft_in_manager() {
            let mut m = ConsensusManager::new();
            m.register(Box::new(RuVectorRaftConsensus::new(make_nodes(3))));
            assert!(m.propose(ConsensusType::Raft, b"v".to_vec()).await.unwrap());
        }
    }
}
