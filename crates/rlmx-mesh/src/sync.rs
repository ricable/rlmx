//! State synchronization between mesh devices.
//!
//! Manages sync state for each device pair, transport selection,
//! conflict resolution policies, and sync health tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::device::DeviceId;

/// Transport protocol used for device-to-device synchronization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyncTransport {
    /// QUIC -- sub-millisecond LAN sync (hub <-> laptop).
    Quic,
    /// WebSocket -- phone <-> cloud relay via MCP WS server (:3001).
    WebSocket,
    /// HTTP -- fallback for cloud burst nodes behind firewalls.
    Http,
    /// BroadcastChannel -- browser tab sync (same origin).
    BroadcastChannel,
}

/// Protocol used for state consistency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyncProtocol {
    /// Raft consensus -- critical state (agent placement, capability tokens).
    RaftConsensus,
    /// CRDT -- eventual consistency on metrics, counters, engagement scores.
    Crdt,
    /// Last-writer-wins -- simple timestamp-based conflict resolution.
    LastWriterWins,
}

/// What category of data is being synced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyncScope {
    AgentPlacement,
    CapabilityTokens,
    SonaPatterns,
    EngagementState,
    EphemeralMetrics,
}

/// Priority level for sync operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyncPriority {
    /// Must sync before operations proceed.
    Critical,
    /// Sync on schedule.
    Normal,
    /// Sync when bandwidth available.
    BestEffort,
}

/// Health status of sync between a device pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyncHealth {
    /// Last sync within threshold.
    Healthy,
    /// Pending ops above warning threshold.
    Degraded,
    /// No sync within timeout.
    Disconnected,
}

/// Policy governing how a particular scope of data synchronizes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPolicy {
    pub scope: SyncScope,
    pub protocol: SyncProtocol,
    pub interval_ms: u64,
    pub priority: SyncPriority,
    pub max_pending: u64,
}

impl SyncPolicy {
    /// Default policies for all sync scopes per ADR-022.
    pub fn defaults() -> Vec<Self> {
        vec![
            SyncPolicy {
                scope: SyncScope::AgentPlacement,
                protocol: SyncProtocol::RaftConsensus,
                interval_ms: 1_000,
                priority: SyncPriority::Critical,
                max_pending: 10,
            },
            SyncPolicy {
                scope: SyncScope::CapabilityTokens,
                protocol: SyncProtocol::RaftConsensus,
                interval_ms: 5_000,
                priority: SyncPriority::Critical,
                max_pending: 5,
            },
            SyncPolicy {
                scope: SyncScope::SonaPatterns,
                protocol: SyncProtocol::Crdt,
                interval_ms: 30_000,
                priority: SyncPriority::Normal,
                max_pending: 100,
            },
            SyncPolicy {
                scope: SyncScope::EngagementState,
                protocol: SyncProtocol::Crdt,
                interval_ms: 60_000,
                priority: SyncPriority::Normal,
                max_pending: 50,
            },
            SyncPolicy {
                scope: SyncScope::EphemeralMetrics,
                protocol: SyncProtocol::LastWriterWins,
                interval_ms: 10_000,
                priority: SyncPriority::BestEffort,
                max_pending: 200,
            },
        ]
    }
}

/// Synchronization state between a pair of devices.
///
/// Entity: tracks the evolving relationship between two specific devices.
/// Each device pair has independent sync progress, backlog, and health.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub source_device: DeviceId,
    pub target_device: DeviceId,
    pub transport: SyncTransport,
    pub last_sync: DateTime<Utc>,
    pub pending_ops: u64,
    pub health: SyncHealth,
}

impl SyncState {
    /// Create a new sync state for a device pair.
    pub fn new(source_device: DeviceId, target_device: DeviceId, transport: SyncTransport) -> Self {
        Self {
            source_device,
            target_device,
            transport,
            last_sync: Utc::now(),
            pending_ops: 0,
            health: SyncHealth::Healthy,
        }
    }

    /// Record a successful sync, resetting pending ops.
    pub fn mark_synced(&mut self, ops_synced: u64) {
        self.pending_ops = self.pending_ops.saturating_sub(ops_synced);
        self.last_sync = Utc::now();
        self.health = if self.pending_ops == 0 {
            SyncHealth::Healthy
        } else {
            SyncHealth::Degraded
        };
    }

    /// Enqueue pending operations.
    pub fn enqueue(&mut self, count: u64) {
        self.pending_ops = self.pending_ops.saturating_add(count);
    }

    /// Evaluate health based on pending ops threshold.
    pub fn evaluate_health(&mut self, degraded_threshold: u64) {
        self.health = if self.pending_ops == 0 {
            SyncHealth::Healthy
        } else if self.pending_ops > degraded_threshold {
            SyncHealth::Disconnected
        } else {
            SyncHealth::Degraded
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn sync_state_new_is_healthy() {
        let ss = SyncState::new(Uuid::new_v4(), Uuid::new_v4(), SyncTransport::Quic);
        assert_eq!(ss.pending_ops, 0);
        assert_eq!(ss.health, SyncHealth::Healthy);
    }

    #[test]
    fn sync_state_enqueue_and_sync() {
        let mut ss = SyncState::new(Uuid::new_v4(), Uuid::new_v4(), SyncTransport::WebSocket);
        ss.enqueue(5);
        assert_eq!(ss.pending_ops, 5);
        ss.mark_synced(3);
        assert_eq!(ss.pending_ops, 2);
        assert_eq!(ss.health, SyncHealth::Degraded);
        ss.mark_synced(2);
        assert_eq!(ss.pending_ops, 0);
        assert_eq!(ss.health, SyncHealth::Healthy);
    }

    #[test]
    fn sync_state_evaluate_health() {
        let mut ss = SyncState::new(Uuid::new_v4(), Uuid::new_v4(), SyncTransport::Http);
        ss.enqueue(50);
        ss.evaluate_health(30);
        assert_eq!(ss.health, SyncHealth::Disconnected);
        ss.mark_synced(30);
        ss.evaluate_health(30);
        assert_eq!(ss.health, SyncHealth::Degraded);
    }

    #[test]
    fn sync_policy_defaults_cover_all_scopes() {
        let policies = SyncPolicy::defaults();
        assert_eq!(policies.len(), 5);
        let scopes: Vec<_> = policies.iter().map(|p| p.scope).collect();
        assert!(scopes.contains(&SyncScope::AgentPlacement));
        assert!(scopes.contains(&SyncScope::CapabilityTokens));
        assert!(scopes.contains(&SyncScope::SonaPatterns));
        assert!(scopes.contains(&SyncScope::EngagementState));
        assert!(scopes.contains(&SyncScope::EphemeralMetrics));
    }

    #[test]
    fn sync_state_serialization_roundtrip() {
        let ss = SyncState::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            SyncTransport::BroadcastChannel,
        );
        let json = serde_json::to_string(&ss).unwrap();
        let back: SyncState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.transport, SyncTransport::BroadcastChannel);
        assert_eq!(back.health, SyncHealth::Healthy);
    }
}
