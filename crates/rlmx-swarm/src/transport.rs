use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::types::{NodeId, SwarmError, SwarmEvent};

/// Payload variants for transport messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    Heartbeat,
    ConsensusProposal(Vec<u8>),
    ConsensusVote { round: u64, approve: bool },
    AgentMessage(serde_json::Value),
    DataSync(Vec<u8>),
    EventBroadcast(SwarmEvent),
}

/// A message sent between swarm nodes.
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

/// Trait for swarm transport implementations.
#[async_trait]
pub trait SwarmTransport: Send + Sync {
    /// Send a message to a specific target node.
    async fn send(&self, target: NodeId, message: TransportMessage) -> Result<(), SwarmError>;

    /// Broadcast a message to all nodes.
    async fn broadcast(&self, message: TransportMessage) -> Result<(), SwarmError>;

    /// Receive the next incoming message.
    async fn receive(&mut self) -> Result<TransportMessage, SwarmError>;
}

/// In-memory transport for simulation mode (uses tokio mpsc channels).
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

    /// Get a clone of the sender (for distributing to simulated nodes).
    pub fn sender(&self) -> mpsc::Sender<TransportMessage> {
        self.tx.clone()
    }

    /// How many messages have been sent through this transport.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_message_creation() {
        let msg = TransportMessage::new(
            NodeId::new(),
            Some(NodeId::new()),
            MessagePayload::Heartbeat,
        );
        assert!(msg.to.is_some());
    }

    #[test]
    fn test_broadcast_message_no_target() {
        let msg = TransportMessage::new(NodeId::new(), None, MessagePayload::Heartbeat);
        assert!(msg.to.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_send_receive() {
        let mut transport = InMemoryTransport::new(16);
        let from = NodeId::new();
        let msg = TransportMessage::new(from, None, MessagePayload::Heartbeat);
        let target = NodeId::new();

        transport.send(target, msg).await.unwrap();
        let received = transport.receive().await.unwrap();
        assert_eq!(received.from, from);
    }

    #[tokio::test]
    async fn test_in_memory_broadcast() {
        let mut transport = InMemoryTransport::new(16);
        let from = NodeId::new();
        let msg = TransportMessage::new(
            from,
            None,
            MessagePayload::ConsensusVote {
                round: 1,
                approve: true,
            },
        );

        transport.broadcast(msg).await.unwrap();
        let received = transport.receive().await.unwrap();
        match received.payload {
            MessagePayload::ConsensusVote { round, approve } => {
                assert_eq!(round, 1);
                assert!(approve);
            }
            _ => panic!("unexpected payload"),
        }
    }

    #[test]
    fn test_message_payload_serialize() {
        let payload = MessagePayload::DataSync(vec![1, 2, 3]);
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("DataSync"));
    }
}
