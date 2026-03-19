//! Graceful degradation and failover policies for the personal mesh.
//!
//! Defines how the mesh reacts when devices go offline, including
//! zone promotion, agent migration, and degradation levels.

use serde::{Deserialize, Serialize};

use crate::device::Zone;

/// Level of service degradation when devices are unavailable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DegradationLevel {
    /// All devices online, full capability.
    Full,
    /// Some devices offline, reduced but functional.
    Reduced,
    /// Only local device available, no sync.
    Offline,
    /// Operating from cached data only, no new computations.
    CacheOnly,
}

/// Reason for a zone failover event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FailoverReason {
    DeviceOffline,
    BatteryLow,
    NetworkDegraded,
    Overloaded,
    UserRequested,
}

/// Reason an agent was migrated between devices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MigrationReason {
    DeviceOffline,
    BatteryLow,
    CapabilityMismatch,
    LoadBalancing,
    UserRequested,
}

/// Policy dictating behavior when a specific zone goes offline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailoverPolicy {
    pub zone: Zone,
    pub fallback_zone: Option<Zone>,
    pub degradation: DegradationLevel,
    pub migrate_agents: bool,
    pub description: String,
}

impl FailoverPolicy {
    /// Default failover policies per ADR-022 degradation table.
    pub fn defaults() -> Vec<Self> {
        vec![
            FailoverPolicy {
                zone: Zone::AMobile,
                fallback_zone: Some(Zone::BCloud),
                degradation: DegradationLevel::Reduced,
                migrate_agents: false,
                description:
                    "Phone offline: 5 always-on WASM agents continue with local SONA cache".into(),
            },
            FailoverPolicy {
                zone: Zone::ADesktop,
                fallback_zone: Some(Zone::BCloud),
                degradation: DegradationLevel::Reduced,
                migrate_agents: true,
                description: "Laptop offline: phone routes to cloud burst for complex queries"
                    .into(),
            },
            FailoverPolicy {
                zone: Zone::CEdge,
                fallback_zone: None,
                degradation: DegradationLevel::Offline,
                migrate_agents: false,
                description:
                    "Home hub offline: devices use local cache, queue syncs in OfflineOutbox".into(),
            },
            FailoverPolicy {
                zone: Zone::BCloud,
                fallback_zone: Some(Zone::ADesktop),
                degradation: DegradationLevel::Reduced,
                migrate_agents: false,
                description: "Cloud offline: all local inference, no federation updates".into(),
            },
            FailoverPolicy {
                zone: Zone::DBrowser,
                fallback_zone: None,
                degradation: DegradationLevel::CacheOnly,
                migrate_agents: false,
                description:
                    "Browser tab closed: stateless workers terminated, no migration needed".into(),
            },
        ]
    }

    /// Look up the failover policy for a given zone from defaults.
    pub fn for_zone(zone: Zone) -> Self {
        Self::defaults()
            .into_iter()
            .find(|p| p.zone == zone)
            .expect("all zones have a default failover policy")
    }
}

/// Result of evaluating mesh degradation across all devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshDegradation {
    pub level: DegradationLevel,
    pub offline_zones: Vec<Zone>,
    pub available_zones: Vec<Zone>,
    pub description: String,
}

/// Evaluate the overall degradation level given the set of online zones.
pub fn evaluate_degradation(online_zones: &[Zone]) -> MeshDegradation {
    if online_zones.is_empty() {
        return MeshDegradation {
            level: DegradationLevel::CacheOnly,
            offline_zones: vec![
                Zone::ADesktop,
                Zone::AMobile,
                Zone::BCloud,
                Zone::CEdge,
                Zone::DBrowser,
            ],
            available_zones: vec![],
            description: "All devices offline: operating from cached patterns only".into(),
        };
    }

    let all_zones = [
        Zone::ADesktop,
        Zone::AMobile,
        Zone::BCloud,
        Zone::CEdge,
        Zone::DBrowser,
    ];
    let offline: Vec<Zone> = all_zones
        .iter()
        .filter(|z| !online_zones.contains(z))
        .copied()
        .collect();

    let level = if offline.is_empty() {
        DegradationLevel::Full
    } else if online_zones.iter().any(|z| z.is_coordinator_zone()) {
        DegradationLevel::Reduced
    } else {
        DegradationLevel::Offline
    };

    let desc = match level {
        DegradationLevel::Full => "All zones operational".into(),
        DegradationLevel::Reduced => format!(
            "Reduced: {} zone(s) offline, coordinator available",
            offline.len()
        ),
        DegradationLevel::Offline => format!(
            "Offline: coordinator unavailable, {} zone(s) online",
            online_zones.len()
        ),
        DegradationLevel::CacheOnly => "Cache only: no zones available".into(),
    };

    MeshDegradation {
        level,
        offline_zones: offline,
        available_zones: online_zones.to_vec(),
        description: desc,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failover_defaults_cover_all_zones() {
        let policies = FailoverPolicy::defaults();
        assert_eq!(policies.len(), 5);
        let zones: Vec<_> = policies.iter().map(|p| p.zone).collect();
        assert!(zones.contains(&Zone::ADesktop));
        assert!(zones.contains(&Zone::AMobile));
        assert!(zones.contains(&Zone::BCloud));
        assert!(zones.contains(&Zone::CEdge));
        assert!(zones.contains(&Zone::DBrowser));
    }

    #[test]
    fn failover_for_zone_laptop() {
        let policy = FailoverPolicy::for_zone(Zone::ADesktop);
        assert_eq!(policy.fallback_zone, Some(Zone::BCloud));
        assert!(policy.migrate_agents);
    }

    #[test]
    fn failover_for_zone_home_hub_no_fallback() {
        let policy = FailoverPolicy::for_zone(Zone::CEdge);
        assert!(policy.fallback_zone.is_none());
        assert_eq!(policy.degradation, DegradationLevel::Offline);
    }

    #[test]
    fn evaluate_degradation_all_online() {
        let deg = evaluate_degradation(&[
            Zone::ADesktop,
            Zone::AMobile,
            Zone::BCloud,
            Zone::CEdge,
            Zone::DBrowser,
        ]);
        assert_eq!(deg.level, DegradationLevel::Full);
        assert!(deg.offline_zones.is_empty());
    }

    #[test]
    fn evaluate_degradation_coordinator_online() {
        let deg = evaluate_degradation(&[Zone::ADesktop, Zone::AMobile]);
        assert_eq!(deg.level, DegradationLevel::Reduced);
        assert_eq!(deg.offline_zones.len(), 3);
    }

    #[test]
    fn evaluate_degradation_no_coordinator() {
        let deg = evaluate_degradation(&[Zone::AMobile, Zone::CEdge]);
        assert_eq!(deg.level, DegradationLevel::Offline);
    }

    #[test]
    fn evaluate_degradation_all_offline() {
        let deg = evaluate_degradation(&[]);
        assert_eq!(deg.level, DegradationLevel::CacheOnly);
        assert_eq!(deg.offline_zones.len(), 5);
    }
}
