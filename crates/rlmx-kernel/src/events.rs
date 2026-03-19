//! Domain events that flow between bounded contexts.
//!
//! See DDD-001-context-map.md for the full event specification.
//! These events are emitted by kernel operations and consumed by other
//! subsystems (Observation & Health, Swarm Coordination, Container & Storage).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Domain events that flow between bounded contexts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    /// Emitted after any syscall is dispatched.
    /// Consumer: Observation & Health (DAG optimizer records execution).
    SyscallDispatched {
        syscall_type: String,
        process_id: u64,
        timestamp: DateTime<Utc>,
        success: bool,
    },
    /// Emitted when a new agent process is forked.
    /// Consumer: Swarm Coordination (updates node agent count).
    AgentSpawned {
        agent_id: Uuid,
        agent_type: String,
        parent_id: Option<Uuid>,
        timestamp: DateTime<Utc>,
    },
    /// Emitted when a query is routed to a strategy.
    /// Consumer: Observation & Health (SONA learns from routing).
    QueryRouted {
        query_hash: u64,
        strategy: String,
        confidence: f64,
        timestamp: DateTime<Utc>,
    },
    /// Emitted when an experiment completes.
    /// Consumer: Observation & Health (records fitness).
    ExperimentCompleted {
        experiment_id: Uuid,
        fitness: f64,
        generation: u32,
        timestamp: DateTime<Utc>,
    },
    /// Emitted when a node joins or leaves the swarm.
    /// Consumer: Agent Lifecycle (rebalances agents).
    NodeMembershipChanged {
        node_id: Uuid,
        joined: bool,
        zone: String,
        timestamp: DateTime<Utc>,
    },
    /// Emitted when state is mutated via StateMutate syscall.
    /// Consumer: Container & Storage (appends to witness chain).
    StateMutated {
        witness_id: Uuid,
        success: bool,
        timestamp: DateTime<Utc>,
    },
    /// Emitted when a voice session begins (ADR-019).
    /// Consumer: Observation & Health (tracks voice session metrics).
    VoiceSessionStarted {
        session_id: Uuid,
        mode: String,
        timestamp: DateTime<Utc>,
    },
    /// Emitted when a transcript is decomposed into intents (ADR-019).
    /// Consumer: Agent Lifecycle (routes intents to domain agents).
    IntentsDecomposed {
        session_id: Uuid,
        intent_count: usize,
        domains: Vec<String>,
        timestamp: DateTime<Utc>,
    },
    /// Emitted when a device joins the personal mesh (ADR-022).
    /// Consumer: Swarm Coordination (updates zone topology).
    MeshDeviceJoined {
        mesh_id: Uuid,
        device_id: Uuid,
        zone: String,
        timestamp: DateTime<Utc>,
    },
    /// Emitted when a federated learning cycle completes (ADR-023).
    /// Consumer: Cognitive (updates SONA with federated patterns).
    FederationCycleCompleted {
        cycle_id: Uuid,
        contributors: usize,
        patterns_aggregated: usize,
        timestamp: DateTime<Utc>,
    },

    // --- AgentOS Cherry-Pick Integration (ADR-030 through ADR-039) ---
    // Artifact, Channel, Board, Budget, and Evolve events are now crate-local
    // (see respective crate types.rs / board.rs / budget.rs):
    //   - ArtifactEvent       in rlmx-artifact::types
    //   - BoardEvent           in rlmx-swarm::board
    //   - BudgetEvent          in rlmx-billing::budget
    //   - EvolveEvent          in rlmx-evolve::types
    //   - ChannelEvent         in rlmx-channels::types

    /// Emitted when human approval is requested (ADR-037).
    /// Consumer: All UI channels (mobile, MCP, WebSocket, CLI).
    ApprovalRequested {
        operation: String,
        agent_id: Uuid,
        tier: String,
        cost_estimate: Option<u64>,
        timestamp: DateTime<Utc>,
    },
    /// Emitted when a human decides on an approval request (ADR-037).
    /// Consumer: Agent Lifecycle (unblocks or denies operation).
    ApprovalDecided {
        operation: String,
        approved: bool,
        decided_by: String,
        timestamp: DateTime<Utc>,
    },
}

/// Event bus type alias for domain event broadcasting.
pub type DomainEventBus = tokio::sync::broadcast::Sender<DomainEvent>;

/// Create a new domain event bus with default capacity (1024 events).
pub fn create_event_bus() -> (
    DomainEventBus,
    tokio::sync::broadcast::Receiver<DomainEvent>,
) {
    tokio::sync::broadcast::channel(1024)
}

/// Helper to emit an event on an optional bus, ignoring send failures
/// (e.g., when there are no active receivers).
pub(crate) fn emit(bus: &Option<DomainEventBus>, event: DomainEvent) {
    if let Some(tx) = bus {
        // Ignore errors: no receivers is acceptable.
        let _ = tx.send(event);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_event_bus() {
        let (tx, _rx) = create_event_bus();
        let event = DomainEvent::SyscallDispatched {
            syscall_type: "VecInsert".into(),
            process_id: 42,
            timestamp: Utc::now(),
            success: true,
        };
        // Should not panic even if no additional receivers.
        let _ = tx.send(event);
    }

    #[test]
    fn test_event_bus_subscription() {
        let (tx, mut rx) = create_event_bus();
        let event = DomainEvent::StateMutated {
            witness_id: Uuid::new_v4(),
            success: true,
            timestamp: Utc::now(),
        };
        tx.send(event.clone()).unwrap();
        let received = rx.try_recv().unwrap();
        assert!(matches!(
            received,
            DomainEvent::StateMutated { success: true, .. }
        ));
    }

    #[test]
    fn test_event_serialization_roundtrip() {
        let events = vec![
            DomainEvent::SyscallDispatched {
                syscall_type: "GraphQuery".into(),
                process_id: 1,
                timestamp: Utc::now(),
                success: true,
            },
            DomainEvent::AgentSpawned {
                agent_id: Uuid::new_v4(),
                agent_type: "coder".into(),
                parent_id: Some(Uuid::new_v4()),
                timestamp: Utc::now(),
            },
            DomainEvent::QueryRouted {
                query_hash: 12345,
                strategy: "Rlm".into(),
                confidence: 0.85,
                timestamp: Utc::now(),
            },
            DomainEvent::ExperimentCompleted {
                experiment_id: Uuid::new_v4(),
                fitness: 0.92,
                generation: 5,
                timestamp: Utc::now(),
            },
            DomainEvent::NodeMembershipChanged {
                node_id: Uuid::new_v4(),
                joined: true,
                zone: "zone-a".into(),
                timestamp: Utc::now(),
            },
            DomainEvent::StateMutated {
                witness_id: Uuid::new_v4(),
                success: false,
                timestamp: Utc::now(),
            },
            DomainEvent::VoiceSessionStarted {
                session_id: Uuid::new_v4(),
                mode: "multimodal".into(),
                timestamp: Utc::now(),
            },
            DomainEvent::IntentsDecomposed {
                session_id: Uuid::new_v4(),
                intent_count: 3,
                domains: vec!["Finance".into(), "Health".into()],
                timestamp: Utc::now(),
            },
            DomainEvent::MeshDeviceJoined {
                mesh_id: Uuid::new_v4(),
                device_id: Uuid::new_v4(),
                zone: "A-Desktop".into(),
                timestamp: Utc::now(),
            },
            DomainEvent::FederationCycleCompleted {
                cycle_id: Uuid::new_v4(),
                contributors: 1500,
                patterns_aggregated: 42000,
                timestamp: Utc::now(),
            },
        ];

        for event in &events {
            let json = serde_json::to_string(event).unwrap();
            let deserialized: DomainEvent = serde_json::from_str(&json).unwrap();
            let json2 = serde_json::to_string(&deserialized).unwrap();
            assert_eq!(json, json2);
        }
    }

    #[test]
    fn test_emit_helper_with_bus() {
        let (tx, mut rx) = create_event_bus();
        let bus = Some(tx);
        emit(
            &bus,
            DomainEvent::StateMutated {
                witness_id: Uuid::new_v4(),
                success: true,
                timestamp: Utc::now(),
            },
        );
        assert!(rx.try_recv().is_ok());
    }

    #[test]
    fn test_emit_helper_without_bus() {
        let bus: Option<DomainEventBus> = None;
        // Should not panic.
        emit(
            &bus,
            DomainEvent::SyscallDispatched {
                syscall_type: "VecSearch".into(),
                process_id: 0,
                timestamp: Utc::now(),
                success: true,
            },
        );
    }
}
