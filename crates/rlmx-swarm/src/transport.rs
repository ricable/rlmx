use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::types::{NodeId, SwarmError, SwarmEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    Heartbeat,
    ConsensusProposal(Vec<u8>),
    ConsensusVote { round: u64, approve: bool },
    AgentMessage(serde_json::Value),
    DataSync(Vec<u8>),
    EventBroadcast(SwarmEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportMessage {
    pub id: Uuid,
    pub from: NodeId,
    pub to: Option<NodeId>,
    pub payload: MessagePayload,
    pub timestamp: DateTime<Utc>,
}

impl TransportMessage {
    pub fn new(from: NodeId, to: Option<NodeId>, payload: MessagePayload) -> Self {
        Self {
            id: Uuid::new_v4(),
            from,
            to,
            payload,
            timestamp: Utc::now(),
        }
    }
}

#[async_trait]
pub trait SwarmTransport: Send + Sync {
    async fn send(&self, target: NodeId, message: TransportMessage) -> Result<(), SwarmError>;
    async fn broadcast(&self, message: TransportMessage) -> Result<(), SwarmError>;
    async fn receive(&mut self) -> Result<TransportMessage, SwarmError>;
}

pub struct InMemoryTransport {
    tx: mpsc::Sender<TransportMessage>,
    rx: mpsc::Receiver<TransportMessage>,
    sent_count: u64,
}

impl InMemoryTransport {
    pub fn new(buffer_size: usize) -> Self {
        let (tx, rx) = mpsc::channel(buffer_size);
        Self {
            tx,
            rx,
            sent_count: 0,
        }
    }
    pub fn sender(&self) -> mpsc::Sender<TransportMessage> {
        self.tx.clone()
    }
    pub fn sent_count(&self) -> u64 {
        self.sent_count
    }
}

#[async_trait]
impl SwarmTransport for InMemoryTransport {
    async fn send(&self, _target: NodeId, message: TransportMessage) -> Result<(), SwarmError> {
        self.tx
            .send(message)
            .await
            .map_err(|e| SwarmError::TransportError(format!("send failed: {e}")))?;
        Ok(())
    }
    async fn broadcast(&self, message: TransportMessage) -> Result<(), SwarmError> {
        self.tx
            .send(message)
            .await
            .map_err(|e| SwarmError::TransportError(format!("broadcast failed: {e}")))?;
        Ok(())
    }
    async fn receive(&mut self) -> Result<TransportMessage, SwarmError> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| SwarmError::TransportError("channel closed".into()))
    }
}

// ---------------------------------------------------------------------------
// Feature-gated: ruv-swarm mesh transport (ADR-024)
// ---------------------------------------------------------------------------
#[cfg(feature = "ruv-swarm")]
pub mod mesh_transport {
    use super::*;
    use ruv_swarm_transport::TransportConfig as RuvTransportConfig;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum MeshTransport {
        Quic,
        WebSocket,
        Http,
        BroadcastChannel,
        InProcess,
        SharedMemory,
    }

    impl std::fmt::Display for MeshTransport {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::Quic => write!(f, "quic"),
                Self::WebSocket => write!(f, "websocket"),
                Self::Http => write!(f, "http"),
                Self::BroadcastChannel => write!(f, "broadcast-channel"),
                Self::InProcess => write!(f, "in-process"),
                Self::SharedMemory => write!(f, "shared-memory"),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct TransportManagerConfig {
        pub max_message_size: usize,
        pub connection_timeout_ms: u64,
        pub retry_attempts: u32,
        pub enable_compression: bool,
        pub preferred_transports: Vec<MeshTransport>,
    }
    impl Default for TransportManagerConfig {
        fn default() -> Self {
            Self {
                max_message_size: 10 * 1024 * 1024,
                connection_timeout_ms: 5000,
                retry_attempts: 3,
                enable_compression: true,
                preferred_transports: vec![
                    MeshTransport::InProcess,
                    MeshTransport::SharedMemory,
                    MeshTransport::WebSocket,
                    MeshTransport::Quic,
                    MeshTransport::Http,
                ],
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct PeerConnection {
        pub node_id: NodeId,
        pub transport: MeshTransport,
        pub address: String,
        pub connected: bool,
        pub messages_sent: u64,
        pub messages_received: u64,
    }

    pub struct TransportManager {
        config: TransportManagerConfig,
        peers: Arc<Mutex<HashMap<NodeId, PeerConnection>>>,
        available_transports: Vec<MeshTransport>,
    }

    impl TransportManager {
        pub fn new() -> Self {
            Self::with_config(TransportManagerConfig::default())
        }
        pub fn with_config(config: TransportManagerConfig) -> Self {
            Self {
                available_transports: config.preferred_transports.clone(),
                config,
                peers: Arc::new(Mutex::new(HashMap::new())),
            }
        }
        pub fn register_peer(&self, node_id: NodeId, transport: MeshTransport, address: String) {
            self.peers.lock().unwrap().insert(
                node_id,
                PeerConnection {
                    node_id,
                    transport,
                    address,
                    connected: true,
                    messages_sent: 0,
                    messages_received: 0,
                },
            );
        }
        pub fn remove_peer(&self, node_id: &NodeId) -> bool {
            self.peers.lock().unwrap().remove(node_id).is_some()
        }
        pub fn get_peer(&self, node_id: &NodeId) -> Option<PeerConnection> {
            self.peers.lock().unwrap().get(node_id).cloned()
        }
        pub fn connected_peers(&self) -> Vec<PeerConnection> {
            self.peers
                .lock()
                .unwrap()
                .values()
                .filter(|p| p.connected)
                .cloned()
                .collect()
        }
        pub fn peer_count(&self) -> usize {
            self.peers.lock().unwrap().len()
        }
        pub fn select_transport(&self, node_id: &NodeId) -> Option<MeshTransport> {
            let peers = self.peers.lock().unwrap();
            if let Some(peer) = peers.get(node_id) {
                if peer.connected {
                    return Some(peer.transport);
                }
            }
            self.available_transports.first().copied()
        }
        pub fn send_to(
            &self,
            target: &NodeId,
            _message: &TransportMessage,
        ) -> Result<(), SwarmError> {
            let mut peers = self.peers.lock().unwrap();
            if let Some(peer) = peers.get_mut(target) {
                if !peer.connected {
                    return Err(SwarmError::TransportError(format!(
                        "peer {} not connected",
                        target
                    )));
                }
                peer.messages_sent += 1;
                Ok(())
            } else {
                Err(SwarmError::NodeNotFound(*target))
            }
        }
        pub fn broadcast(&self, _message: &TransportMessage) -> Result<usize, SwarmError> {
            let mut peers = self.peers.lock().unwrap();
            let mut sent = 0;
            for peer in peers.values_mut() {
                if peer.connected {
                    peer.messages_sent += 1;
                    sent += 1;
                }
            }
            Ok(sent)
        }
        pub fn ruv_transport_config(&self) -> RuvTransportConfig {
            RuvTransportConfig {
                max_message_size: self.config.max_message_size,
                connection_timeout_ms: self.config.connection_timeout_ms,
                retry_attempts: self.config.retry_attempts,
                enable_compression: self.config.enable_compression,
                compression_threshold: 1024,
            }
        }
        pub fn available_transports(&self) -> &[MeshTransport] {
            &self.available_transports
        }
    }
    impl Default for TransportManager {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_message_creation() {
        let m = TransportMessage::new(
            NodeId::new(),
            Some(NodeId::new()),
            MessagePayload::Heartbeat,
        );
        assert!(m.to.is_some());
    }
    #[test]
    fn test_broadcast_message_no_target() {
        let m = TransportMessage::new(NodeId::new(), None, MessagePayload::Heartbeat);
        assert!(m.to.is_none());
    }
    #[tokio::test]
    async fn test_in_memory_send_receive() {
        let mut t = InMemoryTransport::new(16);
        let from = NodeId::new();
        t.send(
            NodeId::new(),
            TransportMessage::new(from, None, MessagePayload::Heartbeat),
        )
        .await
        .unwrap();
        assert_eq!(t.receive().await.unwrap().from, from);
    }
    #[tokio::test]
    async fn test_in_memory_broadcast() {
        let mut t = InMemoryTransport::new(16);
        t.broadcast(TransportMessage::new(
            NodeId::new(),
            None,
            MessagePayload::ConsensusVote {
                round: 1,
                approve: true,
            },
        ))
        .await
        .unwrap();
        match t.receive().await.unwrap().payload {
            MessagePayload::ConsensusVote { round, approve } => {
                assert_eq!(round, 1);
                assert!(approve);
            }
            _ => panic!("unexpected"),
        }
    }
    #[test]
    fn test_message_payload_serialize() {
        let j = serde_json::to_string(&MessagePayload::DataSync(vec![1, 2, 3])).unwrap();
        assert!(j.contains("DataSync"));
    }

    #[cfg(feature = "ruv-swarm")]
    mod mesh_transport_tests {
        use super::*;
        use crate::transport::mesh_transport::{
            MeshTransport, TransportManager, TransportManagerConfig,
        };
        #[test]
        fn test_register_and_count() {
            let m = TransportManager::new();
            m.register_peer(NodeId::new(), MeshTransport::WebSocket, "ws://a".into());
            m.register_peer(NodeId::new(), MeshTransport::InProcess, "l://0".into());
            assert_eq!(m.peer_count(), 2);
        }
        #[test]
        fn test_remove_peer() {
            let m = TransportManager::new();
            let n = NodeId::new();
            m.register_peer(n, MeshTransport::Quic, "q://1".into());
            assert!(m.remove_peer(&n));
            assert!(!m.remove_peer(&n));
            assert_eq!(m.peer_count(), 0);
        }
        #[test]
        fn test_select_transport() {
            let m = TransportManager::new();
            let n = NodeId::new();
            m.register_peer(n, MeshTransport::SharedMemory, "shm://7".into());
            assert_eq!(m.select_transport(&n), Some(MeshTransport::SharedMemory));
            assert_eq!(
                m.select_transport(&NodeId::new()),
                Some(MeshTransport::InProcess)
            );
        }
        #[test]
        fn test_send_to_counter() {
            let m = TransportManager::new();
            let n = NodeId::new();
            m.register_peer(n, MeshTransport::Http, "h://8080".into());
            let msg = TransportMessage::new(NodeId::new(), Some(n), MessagePayload::Heartbeat);
            m.send_to(&n, &msg).unwrap();
            m.send_to(&n, &msg).unwrap();
            assert_eq!(m.get_peer(&n).unwrap().messages_sent, 2);
        }
        #[test]
        fn test_broadcast_count() {
            let m = TransportManager::new();
            m.register_peer(NodeId::new(), MeshTransport::WebSocket, "a".into());
            m.register_peer(NodeId::new(), MeshTransport::WebSocket, "b".into());
            m.register_peer(NodeId::new(), MeshTransport::WebSocket, "c".into());
            assert_eq!(
                m.broadcast(&TransportMessage::new(
                    NodeId::new(),
                    None,
                    MessagePayload::Heartbeat
                ))
                .unwrap(),
                3
            );
        }
        #[test]
        fn test_send_unknown() {
            assert!(TransportManager::new()
                .send_to(
                    &NodeId::new(),
                    &TransportMessage::new(NodeId::new(), None, MessagePayload::Heartbeat)
                )
                .is_err());
        }
        #[test]
        fn test_display() {
            assert_eq!(MeshTransport::Quic.to_string(), "quic");
            assert_eq!(MeshTransport::WebSocket.to_string(), "websocket");
            assert_eq!(
                MeshTransport::BroadcastChannel.to_string(),
                "broadcast-channel"
            );
        }
        #[test]
        fn test_ruv_config() {
            let c = TransportManagerConfig {
                max_message_size: 1024,
                connection_timeout_ms: 2000,
                retry_attempts: 5,
                enable_compression: false,
                preferred_transports: vec![MeshTransport::Quic],
            };
            let r = TransportManager::with_config(c).ruv_transport_config();
            assert_eq!(r.max_message_size, 1024);
            assert_eq!(r.retry_attempts, 5);
            assert!(!r.enable_compression);
        }
    }
}
