use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::AgentId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncState {
    InSync,
    Syncing,
    Behind(u64),
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub zone_pair: (String, String),
    pub last_sync: DateTime<Utc>,
    pub items_synced: u64,
    pub status: SyncState,
}

/// Replicator agent — synchronizes data between zones.
pub struct ReplicatorAgent {
    pub id: AgentId,
    sync_status: HashMap<String, SyncStatus>,
}

impl ReplicatorAgent {
    pub fn new() -> Self {
        Self {
            id: AgentId::new(),
            sync_status: HashMap::new(),
        }
    }

    /// Start or update a sync operation between two zones.
    pub fn start_sync(&mut self, from_zone: impl Into<String>, to_zone: impl Into<String>) {
        let from = from_zone.into();
        let to = to_zone.into();
        let key = format!("{}→{}", from, to);

        let status = SyncStatus {
            zone_pair: (from, to),
            last_sync: Utc::now(),
            items_synced: 0,
            status: SyncState::Syncing,
        };

        self.sync_status.insert(key, status);
    }

    /// Mark a sync as completed with item count.
    pub fn complete_sync(&mut self, from_zone: &str, to_zone: &str, items: u64) {
        let key = format!("{from_zone}→{to_zone}");
        if let Some(status) = self.sync_status.get_mut(&key) {
            status.items_synced += items;
            status.last_sync = Utc::now();
            status.status = SyncState::InSync;
        }
    }

    /// Mark a sync as behind by a given number of items.
    pub fn mark_behind(&mut self, from_zone: &str, to_zone: &str, behind_count: u64) {
        let key = format!("{from_zone}→{to_zone}");
        if let Some(status) = self.sync_status.get_mut(&key) {
            status.status = SyncState::Behind(behind_count);
        }
    }

    /// Mark a sync as errored.
    pub fn mark_error(&mut self, from_zone: &str, to_zone: &str, error: impl Into<String>) {
        let key = format!("{from_zone}→{to_zone}");
        if let Some(status) = self.sync_status.get_mut(&key) {
            status.status = SyncState::Error(error.into());
        }
    }

    /// Get all sync statuses.
    pub fn sync_status(&self) -> &HashMap<String, SyncStatus> {
        &self.sync_status
    }

    /// Count total items pending sync across all zone pairs.
    pub fn items_pending(&self) -> u64 {
        self.sync_status
            .values()
            .map(|s| match &s.status {
                SyncState::Behind(n) => *n,
                SyncState::Syncing => 1, // At least one pending
                _ => 0,
            })
            .sum()
    }
}

impl Default for ReplicatorAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_sync() {
        let mut replicator = ReplicatorAgent::new();
        replicator.start_sync("A", "B");
        assert_eq!(replicator.sync_status().len(), 1);
        let status = replicator.sync_status().get("A→B").unwrap();
        assert_eq!(status.zone_pair, ("A".into(), "B".into()));
        assert!(matches!(status.status, SyncState::Syncing));
    }

    #[test]
    fn test_complete_sync() {
        let mut replicator = ReplicatorAgent::new();
        replicator.start_sync("A", "B");
        replicator.complete_sync("A", "B", 100);
        let status = replicator.sync_status().get("A→B").unwrap();
        assert_eq!(status.items_synced, 100);
        assert!(matches!(status.status, SyncState::InSync));
    }

    #[test]
    fn test_mark_behind() {
        let mut replicator = ReplicatorAgent::new();
        replicator.start_sync("A", "C");
        replicator.mark_behind("A", "C", 50);
        let status = replicator.sync_status().get("A→C").unwrap();
        assert!(matches!(status.status, SyncState::Behind(50)));
    }

    #[test]
    fn test_mark_error() {
        let mut replicator = ReplicatorAgent::new();
        replicator.start_sync("B", "C");
        replicator.mark_error("B", "C", "network timeout");
        let status = replicator.sync_status().get("B→C").unwrap();
        assert!(matches!(status.status, SyncState::Error(_)));
    }

    #[test]
    fn test_items_pending() {
        let mut replicator = ReplicatorAgent::new();
        replicator.start_sync("A", "B");
        replicator.start_sync("A", "C");
        replicator.mark_behind("A", "B", 30);
        // A→B: 30 behind, A→C: 1 (syncing)
        assert_eq!(replicator.items_pending(), 31);
    }

    #[test]
    fn test_items_pending_all_synced() {
        let mut replicator = ReplicatorAgent::new();
        replicator.start_sync("A", "B");
        replicator.complete_sync("A", "B", 100);
        assert_eq!(replicator.items_pending(), 0);
    }

    #[test]
    fn test_multiple_syncs_same_pair() {
        let mut replicator = ReplicatorAgent::new();
        replicator.start_sync("A", "B");
        replicator.complete_sync("A", "B", 50);
        replicator.start_sync("A", "B"); // Restart
                                         // Should overwrite
        let status = replicator.sync_status().get("A→B").unwrap();
        assert!(matches!(status.status, SyncState::Syncing));
        assert_eq!(status.items_synced, 0);
    }
}
