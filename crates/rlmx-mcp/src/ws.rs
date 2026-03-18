//! WebSocket Transport
//!
//! Real-time event broadcasting via WebSocket connections (ADR-008).
//! Clients connect to `ws://host:{ws_port}/events` and receive filtered
//! `SwarmEvent` broadcasts. Supports subscription management, heartbeat,
//! backpressure, authentication, and connection limits.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, Mutex, Semaphore};
use tokio::time::{interval, timeout};
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// SwarmEvent types (ADR-008)
// ---------------------------------------------------------------------------

/// Typed events broadcast through the WebSocket event bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum SwarmEvent {
    NodeJoined {
        node_id: Uuid,
        zone: Zone,
        capabilities: Vec<String>,
    },
    NodeLeft {
        node_id: Uuid,
        reason: LeaveReason,
    },
    AgentSpawned {
        agent_id: Uuid,
        agent_type: String,
        node_id: Uuid,
    },
    AgentTerminated {
        agent_id: Uuid,
        reason: TerminationReason,
    },
    HealthUpdate {
        node_id: Uuid,
        cpu: f32,
        mem_mb: u64,
        gpu_util: Option<f32>,
    },
    ExperimentUpdate {
        experiment_id: Uuid,
        generation: u32,
        val_bpb: f64,
        status: ExpStatus,
    },
    MutationFound {
        mutation_id: Uuid,
        fitness: f64,
        generation: u32,
        parent_id: Option<Uuid>,
    },
}

impl SwarmEvent {
    /// Returns the event type name as used in subscription filters.
    pub fn type_name(&self) -> &'static str {
        match self {
            SwarmEvent::NodeJoined { .. } => "NodeJoined",
            SwarmEvent::NodeLeft { .. } => "NodeLeft",
            SwarmEvent::AgentSpawned { .. } => "AgentSpawned",
            SwarmEvent::AgentTerminated { .. } => "AgentTerminated",
            SwarmEvent::HealthUpdate { .. } => "HealthUpdate",
            SwarmEvent::ExperimentUpdate { .. } => "ExperimentUpdate",
            SwarmEvent::MutationFound { .. } => "MutationFound",
        }
    }

    /// Returns the node_id associated with this event, if any.
    fn node_id(&self) -> Option<Uuid> {
        match self {
            SwarmEvent::NodeJoined { node_id, .. }
            | SwarmEvent::NodeLeft { node_id, .. }
            | SwarmEvent::AgentSpawned { node_id, .. }
            | SwarmEvent::HealthUpdate { node_id, .. } => Some(*node_id),
            _ => None,
        }
    }

    /// Returns the zone associated with this event, if any.
    fn zone(&self) -> Option<&Zone> {
        match self {
            SwarmEvent::NodeJoined { zone, .. } => Some(zone),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Zone {
    Mac,
    Nuc,
    Edge,
    Browser,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LeaveReason {
    Graceful,
    Timeout,
    Crashed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TerminationReason {
    Completed,
    Failed,
    Cancelled,
    ResourceLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExpStatus {
    Running,
    Completed,
    Failed,
    CrossPollinated,
}

// ---------------------------------------------------------------------------
// Subscription & filters
// ---------------------------------------------------------------------------

/// Client subscription message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsSubscription {
    pub action: String,
    pub event_types: Vec<String>,
    #[serde(default)]
    pub filters: Option<WsFilters>,
}

/// Per-connection event filters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WsFilters {
    pub node_id: Option<Uuid>,
    pub zone: Option<String>,
}

/// Authentication message -- must be the first message after connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsAuthMessage {
    pub auth_token: String,
}

// ---------------------------------------------------------------------------
// Event bus type alias
// ---------------------------------------------------------------------------

/// The broadcast sender used throughout the system to publish swarm events.
pub type SwarmEventBus = broadcast::Sender<SwarmEvent>;

// ---------------------------------------------------------------------------
// Connection-level state
// ---------------------------------------------------------------------------

/// Per-connection subscription state.
struct ClientState {
    event_types: Vec<String>,
    filters: WsFilters,
}

impl Default for ClientState {
    fn default() -> Self {
        Self {
            event_types: vec![
                "NodeJoined".into(),
                "NodeLeft".into(),
                "AgentSpawned".into(),
                "AgentTerminated".into(),
                "HealthUpdate".into(),
                "ExperimentUpdate".into(),
                "MutationFound".into(),
            ],
            filters: WsFilters::default(),
        }
    }
}

impl ClientState {
    /// Returns `true` if the event matches this client's subscriptions and filters.
    fn matches(&self, event: &SwarmEvent) -> bool {
        // Check event type filter.
        if !self.event_types.is_empty()
            && !self.event_types.contains(&event.type_name().to_string())
        {
            return false;
        }

        // Check node_id filter.
        if let Some(filter_node) = &self.filters.node_id {
            if let Some(event_node) = event.node_id() {
                if &event_node != filter_node {
                    return false;
                }
            }
        }

        // Check zone filter.
        if let Some(filter_zone) = &self.filters.zone {
            if let Some(event_zone) = event.zone() {
                let zone_str = format!("{:?}", event_zone);
                if !zone_str.eq_ignore_ascii_case(filter_zone) {
                    return false;
                }
            }
        }

        true
    }

    /// Apply a subscribe action.
    fn subscribe(&mut self, sub: &WsSubscription) {
        for et in &sub.event_types {
            if !self.event_types.contains(et) {
                self.event_types.push(et.clone());
            }
        }
        if let Some(filters) = &sub.filters {
            self.filters = filters.clone();
        }
    }

    /// Apply an unsubscribe action.
    fn unsubscribe(&mut self, sub: &WsSubscription) {
        self.event_types.retain(|s| !sub.event_types.contains(s));
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum concurrent WebSocket connections.
const MAX_WS_CONNECTIONS: usize = 64;

/// Heartbeat interval in seconds.
const HEARTBEAT_INTERVAL_SECS: u64 = 30;

/// Number of missed pongs before disconnect.
const MAX_MISSED_PONGS: u8 = 3;

/// Maximum pending outbound messages before backpressure disconnect.
const MAX_PENDING_MESSAGES: usize = 256;

/// Timeout for authentication after connection (seconds).
const AUTH_TIMEOUT_SECS: u64 = 5;

// ---------------------------------------------------------------------------
// WsServer
// ---------------------------------------------------------------------------

/// The WebSocket event server.
pub struct WsServer {
    /// Broadcast channel for events.
    event_tx: SwarmEventBus,
    /// Connected client count (for monitoring).
    client_count: Arc<AtomicUsize>,
    /// Connection-limiting semaphore.
    connection_semaphore: Arc<Semaphore>,
    /// Authentication token for validating clients.
    /// When `None`, authentication is disabled.
    auth_token: Option<String>,
}

impl WsServer {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            event_tx,
            client_count: Arc::new(AtomicUsize::new(0)),
            connection_semaphore: Arc::new(Semaphore::new(MAX_WS_CONNECTIONS)),
            auth_token: None,
        }
    }

    /// Create a server with authentication enabled.
    pub fn with_auth(auth_token: String) -> Self {
        let mut server = Self::new();
        server.auth_token = Some(auth_token);
        server
    }

    /// Get a sender handle for broadcasting events.
    pub fn event_sender(&self) -> SwarmEventBus {
        self.event_tx.clone()
    }

    /// Get current connected client count.
    pub fn client_count(&self) -> usize {
        self.client_count.load(Ordering::Relaxed)
    }

    /// Broadcast an event to all connected clients.
    pub fn broadcast(&self, event: SwarmEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Start the WebSocket server on the given host and port.
    pub async fn start(
        self,
        host: &str,
        port: u16,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = format!("{}:{}", host, port);
        let listener = TcpListener::bind(&addr).await?;
        info!(address = %addr, "WebSocket server listening");

        let event_tx = self.event_tx.clone();
        let client_count = self.client_count.clone();
        let semaphore = self.connection_semaphore.clone();
        let auth_token = self.auth_token.clone();

        loop {
            let (stream, peer_addr) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!(error = %e, "Failed to accept WS connection");
                    continue;
                }
            };

            // Connection limit via semaphore.
            let permit = match semaphore.clone().try_acquire_owned() {
                Ok(p) => p,
                Err(_) => {
                    warn!(
                        peer = %peer_addr,
                        "WS connection rejected: limit reached ({} max)",
                        MAX_WS_CONNECTIONS
                    );
                    drop(stream);
                    continue;
                }
            };

            debug!(peer = %peer_addr, "New WebSocket connection");
            let tx = event_tx.clone();
            let count = client_count.clone();
            let token = auth_token.clone();

            tokio::spawn(async move {
                let _permit = permit; // released when task completes

                // Upgrade TCP to WebSocket.
                let ws_stream = match tokio_tungstenite::accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        warn!(error = %e, "WebSocket handshake failed");
                        return;
                    }
                };

                count.fetch_add(1, Ordering::Relaxed);
                info!(peer = %peer_addr, "WebSocket client connected");

                let (mut ws_sender, mut ws_receiver) = ws_stream.split();

                // --- Authentication ---
                if let Some(expected_token) = &token {
                    let auth_result =
                        timeout(Duration::from_secs(AUTH_TIMEOUT_SECS), ws_receiver.next()).await;

                    let authenticated = match auth_result {
                        Ok(Some(Ok(Message::Text(ref text)))) => {
                            serde_json::from_str::<WsAuthMessage>(text)
                                .map(|msg| msg.auth_token == *expected_token)
                                .unwrap_or(false)
                        }
                        _ => false,
                    };

                    if !authenticated {
                        warn!(peer = %peer_addr, "WS authentication failed -- closing");
                        let _ = ws_sender
                            .send(Message::Close(Some(CloseFrame {
                                code: CloseCode::Policy,
                                reason: "Authentication failed".into(),
                            })))
                            .await;
                        count.fetch_sub(1, Ordering::Relaxed);
                        return;
                    }
                    debug!(peer = %peer_addr, "WS client authenticated");
                }

                // --- Per-connection state ---
                let state: Arc<Mutex<ClientState>> =
                    Arc::new(Mutex::new(ClientState::default()));
                let state_for_send = state.clone();

                let mut event_rx = tx.subscribe();

                // Pending message counter for backpressure.
                let pending = Arc::new(AtomicUsize::new(0));
                let pending_for_send = pending.clone();

                // --- Send task: forward filtered events + heartbeat ---
                let send_task = tokio::spawn(async move {
                    let mut heartbeat = interval(Duration::from_secs(HEARTBEAT_INTERVAL_SECS));
                    let mut missed_pongs: u8 = 0;

                    loop {
                        tokio::select! {
                            event_result = event_rx.recv() => {
                                match event_result {
                                    Ok(event) => {
                                        let subs = state_for_send.lock().await;
                                        if !subs.matches(&event) {
                                            continue;
                                        }
                                        drop(subs);

                                        // Backpressure check.
                                        let current_pending =
                                            pending_for_send.load(Ordering::Relaxed);
                                        if current_pending >= MAX_PENDING_MESSAGES {
                                            warn!(
                                                "Backpressure limit reached -- disconnecting client"
                                            );
                                            let _ = ws_sender
                                                .send(Message::Close(Some(CloseFrame {
                                                    code: CloseCode::Policy,
                                                    reason: "Backpressure: too many pending messages"
                                                        .into(),
                                                })))
                                                .await;
                                            break;
                                        }

                                        let json = match serde_json::to_string(&event) {
                                            Ok(j) => j,
                                            Err(_) => continue,
                                        };
                                        pending_for_send.fetch_add(1, Ordering::Relaxed);
                                        if ws_sender.send(Message::Text(json)).await.is_err() {
                                            break;
                                        }
                                        pending_for_send.fetch_sub(1, Ordering::Relaxed);
                                    }
                                    Err(broadcast::error::RecvError::Lagged(n)) => {
                                        warn!(missed = n, "Client lagged behind event stream");
                                    }
                                    Err(_) => break,
                                }
                            }
                            _ = heartbeat.tick() => {
                                if missed_pongs >= MAX_MISSED_PONGS {
                                    warn!(
                                        "Client missed {} pongs -- disconnecting",
                                        missed_pongs
                                    );
                                    let _ = ws_sender
                                        .send(Message::Close(Some(CloseFrame {
                                            code: CloseCode::Away,
                                            reason: "Heartbeat timeout".into(),
                                        })))
                                        .await;
                                    break;
                                }
                                if ws_sender
                                    .send(Message::Ping(vec![].into()))
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                                missed_pongs += 1;
                            }
                        }
                    }
                });

                // --- Receive task: handle subscription messages + pongs ---
                while let Some(msg) = ws_receiver.next().await {
                    match msg {
                        Ok(Message::Text(ref text)) => {
                            if let Ok(sub) = serde_json::from_str::<WsSubscription>(text) {
                                let mut s = state.lock().await;
                                match sub.action.as_str() {
                                    "subscribe" => s.subscribe(&sub),
                                    "unsubscribe" => s.unsubscribe(&sub),
                                    _ => {}
                                }
                            }
                        }
                        Ok(Message::Pong(_)) => {
                            // Pong received -- tungstenite auto-responds to pings,
                            // but we track server-initiated pings via missed_pongs
                            // in the send task.
                        }
                        Ok(Message::Close(_)) => break,
                        Err(_) => break,
                        _ => {}
                    }
                }

                send_task.abort();
                count.fetch_sub(1, Ordering::Relaxed);
                info!(peer = %peer_addr, "WebSocket client disconnected");
            });
        }
    }
}

impl Default for WsServer {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- SwarmEvent serialization tests (all 7 variants) ---

    #[test]
    fn test_serialize_node_joined() {
        let event = SwarmEvent::NodeJoined {
            node_id: Uuid::nil(),
            zone: Zone::Mac,
            capabilities: vec!["gpu".into(), "metal".into()],
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"NodeJoined\""));
        assert!(json.contains("\"zone\":\"Mac\""));
        let deserialized: SwarmEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.type_name(), "NodeJoined");
    }

    #[test]
    fn test_serialize_node_left() {
        let event = SwarmEvent::NodeLeft {
            node_id: Uuid::nil(),
            reason: LeaveReason::Timeout,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"NodeLeft\""));
        let deserialized: SwarmEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.type_name(), "NodeLeft");
    }

    #[test]
    fn test_serialize_agent_spawned() {
        let event = SwarmEvent::AgentSpawned {
            agent_id: Uuid::nil(),
            agent_type: "coder".into(),
            node_id: Uuid::nil(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"AgentSpawned\""));
        assert!(json.contains("\"agent_type\":\"coder\""));
        let deserialized: SwarmEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.type_name(), "AgentSpawned");
    }

    #[test]
    fn test_serialize_agent_terminated() {
        let event = SwarmEvent::AgentTerminated {
            agent_id: Uuid::nil(),
            reason: TerminationReason::ResourceLimit,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"AgentTerminated\""));
        assert!(json.contains("\"ResourceLimit\""));
        let deserialized: SwarmEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.type_name(), "AgentTerminated");
    }

    #[test]
    fn test_serialize_health_update() {
        let event = SwarmEvent::HealthUpdate {
            node_id: Uuid::nil(),
            cpu: 45.5,
            mem_mb: 8192,
            gpu_util: Some(78.0),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"HealthUpdate\""));
        assert!(json.contains("\"gpu_util\":78.0"));
        let deserialized: SwarmEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.type_name(), "HealthUpdate");
    }

    #[test]
    fn test_serialize_experiment_update() {
        let event = SwarmEvent::ExperimentUpdate {
            experiment_id: Uuid::nil(),
            generation: 42,
            val_bpb: 1.234,
            status: ExpStatus::CrossPollinated,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"ExperimentUpdate\""));
        assert!(json.contains("\"CrossPollinated\""));
        let deserialized: SwarmEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.type_name(), "ExperimentUpdate");
    }

    #[test]
    fn test_serialize_mutation_found() {
        let parent = Uuid::new_v4();
        let event = SwarmEvent::MutationFound {
            mutation_id: Uuid::nil(),
            fitness: 0.95,
            generation: 7,
            parent_id: Some(parent),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"MutationFound\""));
        assert!(json.contains("\"fitness\":0.95"));
        let deserialized: SwarmEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.type_name(), "MutationFound");
    }

    #[test]
    fn test_serialize_mutation_found_no_parent() {
        let event = SwarmEvent::MutationFound {
            mutation_id: Uuid::nil(),
            fitness: 0.5,
            generation: 1,
            parent_id: None,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"parent_id\":null"));
    }

    // --- Subscription with filters ---

    #[test]
    fn test_subscription_parse_with_filters() {
        let json = r#"{
            "action": "subscribe",
            "event_types": ["NodeJoined", "HealthUpdate"],
            "filters": {
                "node_id": "00000000-0000-0000-0000-000000000000",
                "zone": "Mac"
            }
        }"#;
        let sub: WsSubscription = serde_json::from_str(json).unwrap();
        assert_eq!(sub.action, "subscribe");
        assert_eq!(sub.event_types.len(), 2);
        let filters = sub.filters.unwrap();
        assert_eq!(filters.node_id, Some(Uuid::nil()));
        assert_eq!(filters.zone, Some("Mac".into()));
    }

    #[test]
    fn test_subscription_parse_without_filters() {
        let json = r#"{"action":"unsubscribe","event_types":["HealthUpdate"]}"#;
        let sub: WsSubscription = serde_json::from_str(json).unwrap();
        assert_eq!(sub.action, "unsubscribe");
        assert!(sub.filters.is_none());
    }

    // --- ClientState filter matching ---

    #[test]
    fn test_client_state_default_matches_all_standard_events() {
        let state = ClientState::default();
        let event = SwarmEvent::NodeJoined {
            node_id: Uuid::nil(),
            zone: Zone::Mac,
            capabilities: vec![],
        };
        assert!(state.matches(&event));
    }

    #[test]
    fn test_client_state_filters_by_event_type() {
        let state = ClientState {
            event_types: vec!["HealthUpdate".into()],
            filters: WsFilters::default(),
        };
        let joined = SwarmEvent::NodeJoined {
            node_id: Uuid::nil(),
            zone: Zone::Mac,
            capabilities: vec![],
        };
        let health = SwarmEvent::HealthUpdate {
            node_id: Uuid::nil(),
            cpu: 50.0,
            mem_mb: 4096,
            gpu_util: None,
        };
        assert!(!state.matches(&joined));
        assert!(state.matches(&health));
    }

    #[test]
    fn test_client_state_filters_by_node_id() {
        let target_node = Uuid::new_v4();
        let other_node = Uuid::new_v4();
        let state = ClientState {
            event_types: vec!["HealthUpdate".into()],
            filters: WsFilters {
                node_id: Some(target_node),
                zone: None,
            },
        };
        let matching = SwarmEvent::HealthUpdate {
            node_id: target_node,
            cpu: 50.0,
            mem_mb: 4096,
            gpu_util: None,
        };
        let non_matching = SwarmEvent::HealthUpdate {
            node_id: other_node,
            cpu: 50.0,
            mem_mb: 4096,
            gpu_util: None,
        };
        assert!(state.matches(&matching));
        assert!(!state.matches(&non_matching));
    }

    #[test]
    fn test_client_state_filters_by_zone() {
        let state = ClientState {
            event_types: vec!["NodeJoined".into()],
            filters: WsFilters {
                node_id: None,
                zone: Some("Mac".into()),
            },
        };
        let mac_event = SwarmEvent::NodeJoined {
            node_id: Uuid::nil(),
            zone: Zone::Mac,
            capabilities: vec![],
        };
        let edge_event = SwarmEvent::NodeJoined {
            node_id: Uuid::nil(),
            zone: Zone::Edge,
            capabilities: vec![],
        };
        assert!(state.matches(&mac_event));
        assert!(!state.matches(&edge_event));
    }

    // --- Subscribe / unsubscribe ---

    #[test]
    fn test_client_state_subscribe() {
        let mut state = ClientState {
            event_types: vec!["NodeJoined".into()],
            filters: WsFilters::default(),
        };
        let sub = WsSubscription {
            action: "subscribe".into(),
            event_types: vec!["HealthUpdate".into(), "NodeJoined".into()],
            filters: None,
        };
        state.subscribe(&sub);
        assert_eq!(state.event_types.len(), 2); // no duplicate
        assert!(state.event_types.contains(&"HealthUpdate".to_string()));
    }

    #[test]
    fn test_client_state_unsubscribe() {
        let mut state = ClientState::default();
        let initial_len = state.event_types.len();
        let sub = WsSubscription {
            action: "unsubscribe".into(),
            event_types: vec!["HealthUpdate".into(), "MutationFound".into()],
            filters: None,
        };
        state.unsubscribe(&sub);
        assert_eq!(state.event_types.len(), initial_len - 2);
        assert!(!state.event_types.contains(&"HealthUpdate".to_string()));
    }

    // --- Connection limit ---

    #[test]
    fn test_connection_limit_semaphore() {
        let sem = Arc::new(Semaphore::new(MAX_WS_CONNECTIONS));
        assert_eq!(sem.available_permits(), 64);

        // Acquire all permits.
        let mut permits = Vec::new();
        for _ in 0..MAX_WS_CONNECTIONS {
            permits.push(sem.clone().try_acquire_owned().unwrap());
        }

        // Next acquire should fail.
        assert!(sem.clone().try_acquire_owned().is_err());

        // Release one -- next acquire should succeed.
        drop(permits.pop());
        assert!(sem.clone().try_acquire_owned().is_ok());
    }

    // --- WsServer basics ---

    #[test]
    fn test_ws_server_creation() {
        let server = WsServer::new();
        let _sender = server.event_sender();
        assert_eq!(server.client_count(), 0);
    }

    #[test]
    fn test_ws_server_with_auth() {
        let server = WsServer::with_auth("secret".into());
        assert!(server.auth_token.is_some());
    }

    #[test]
    fn test_broadcast_no_receivers() {
        let server = WsServer::new();
        server.broadcast(SwarmEvent::NodeLeft {
            node_id: Uuid::nil(),
            reason: LeaveReason::Graceful,
        });
        // Should not panic.
    }

    #[test]
    fn test_event_bus_type_alias() {
        let (tx, mut rx) = broadcast::channel::<SwarmEvent>(16);
        let _: SwarmEventBus = tx.clone();
        let event = SwarmEvent::HealthUpdate {
            node_id: Uuid::nil(),
            cpu: 10.0,
            mem_mb: 1024,
            gpu_util: None,
        };
        tx.send(event.clone()).unwrap();
        let received = rx.try_recv().unwrap();
        assert_eq!(received.type_name(), "HealthUpdate");
    }

    // --- Auth message ---

    #[test]
    fn test_auth_message_parse() {
        let json = r#"{"auth_token":"my-secret-token"}"#;
        let msg: WsAuthMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.auth_token, "my-secret-token");
    }

    // --- Zone/enum roundtrip ---

    #[test]
    fn test_zone_roundtrip() {
        for zone in [Zone::Mac, Zone::Nuc, Zone::Edge, Zone::Browser] {
            let json = serde_json::to_string(&zone).unwrap();
            let back: Zone = serde_json::from_str(&json).unwrap();
            assert_eq!(back, zone);
        }
    }

    #[test]
    fn test_leave_reason_roundtrip() {
        for reason in [
            LeaveReason::Graceful,
            LeaveReason::Timeout,
            LeaveReason::Crashed,
        ] {
            let json = serde_json::to_string(&reason).unwrap();
            let back: LeaveReason = serde_json::from_str(&json).unwrap();
            assert_eq!(back, reason);
        }
    }

    #[test]
    fn test_termination_reason_roundtrip() {
        for reason in [
            TerminationReason::Completed,
            TerminationReason::Failed,
            TerminationReason::Cancelled,
            TerminationReason::ResourceLimit,
        ] {
            let json = serde_json::to_string(&reason).unwrap();
            let back: TerminationReason = serde_json::from_str(&json).unwrap();
            assert_eq!(back, reason);
        }
    }

    #[test]
    fn test_exp_status_roundtrip() {
        for status in [
            ExpStatus::Running,
            ExpStatus::Completed,
            ExpStatus::Failed,
            ExpStatus::CrossPollinated,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: ExpStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(back, status);
        }
    }
}
