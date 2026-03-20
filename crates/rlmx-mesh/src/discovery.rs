//! Device discovery for the personal mesh.
//!
//! Handles mDNS announcement on LAN (home hub <-> laptop) and
//! WebSocket registration for phone/cloud devices.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::device::{DeviceCapabilities, DeviceId, DeviceType};

/// Method by which a device was discovered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// mDNS on local network.
    Mdns,
    /// WebSocket registration via cloud relay.
    WebSocket,
    /// Manual configuration by user.
    Manual,
    /// BroadcastChannel (browser tabs, same origin).
    BroadcastChannel,
}

/// A device announcement broadcast during discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAnnouncement {
    pub device_id: DeviceId,
    pub device_type: DeviceType,
    pub name: String,
    pub capabilities: DeviceCapabilities,
    pub mesh_id: Option<Uuid>,
    pub method: DiscoveryMethod,
    pub endpoint: String,
    pub announced_at: DateTime<Utc>,
}

/// Result of a device join request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinResult {
    pub device_id: DeviceId,
    pub accepted: bool,
    pub assigned_zone: Option<String>,
    pub reason: Option<String>,
}

/// Service responsible for device discovery and mesh join operations.
///
/// In production this would use mDNS for LAN and WebSocket for WAN.
/// The current implementation provides the domain model without
/// transport-specific logic (see ADR-022 for protocol details).
pub struct DiscoveryService {
    /// Known announcements from devices on the network.
    announcements: Vec<DeviceAnnouncement>,
    /// The mesh ID this service is bound to (if any).
    mesh_id: Option<Uuid>,
}

impl DiscoveryService {
    /// Create a new discovery service.
    pub fn new(mesh_id: Option<Uuid>) -> Self {
        Self {
            announcements: Vec::new(),
            mesh_id,
        }
    }

    /// Record a device announcement.
    pub fn announce(&mut self, announcement: DeviceAnnouncement) {
        tracing::info!(
            device_id = %announcement.device_id,
            method = ?announcement.method,
            name = %announcement.name,
            "Device announced on mesh"
        );
        // Replace existing announcement from same device.
        self.announcements
            .retain(|a| a.device_id != announcement.device_id);
        self.announcements.push(announcement);
    }

    /// Return all known device announcements.
    pub fn discover(&self) -> &[DeviceAnnouncement] {
        &self.announcements
    }

    /// Filter announcements to devices not yet in a mesh (available to join).
    pub fn discover_available(&self) -> Vec<&DeviceAnnouncement> {
        self.announcements
            .iter()
            .filter(|a| a.mesh_id.is_none())
            .collect()
    }

    /// Attempt to join the mesh with a device. Returns a join result.
    pub fn join_mesh(&mut self, device_id: DeviceId, mesh_id: Uuid) -> JoinResult {
        if let Some(ann) = self
            .announcements
            .iter_mut()
            .find(|a| a.device_id == device_id)
        {
            if ann.mesh_id.is_some() {
                return JoinResult {
                    device_id,
                    accepted: false,
                    assigned_zone: None,
                    reason: Some("device already in a mesh".into()),
                };
            }
            let zone = ann.device_type.default_zone();
            ann.mesh_id = Some(mesh_id);
            tracing::info!(
                device_id = %device_id,
                mesh_id = %mesh_id,
                zone = ?zone,
                "Device joined mesh"
            );
            JoinResult {
                device_id,
                accepted: true,
                assigned_zone: Some(zone.as_swarm_zone().to_string()),
                reason: None,
            }
        } else {
            JoinResult {
                device_id,
                accepted: false,
                assigned_zone: None,
                reason: Some("device not found in discovery".into()),
            }
        }
    }

    /// Remove a device from the known announcements.
    pub fn remove(&mut self, device_id: DeviceId) {
        self.announcements.retain(|a| a.device_id != device_id);
    }

    /// Get the mesh ID this service is bound to.
    pub fn mesh_id(&self) -> Option<Uuid> {
        self.mesh_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_announcement(name: &str, device_type: DeviceType) -> DeviceAnnouncement {
        DeviceAnnouncement {
            device_id: Uuid::new_v4(),
            device_type,
            name: name.into(),
            capabilities: DeviceCapabilities::default(),
            mesh_id: None,
            method: DiscoveryMethod::Mdns,
            endpoint: "192.168.1.10:3001".into(),
            announced_at: Utc::now(),
        }
    }

    #[test]
    fn announce_and_discover() {
        let mut svc = DiscoveryService::new(None);
        let ann = make_announcement("laptop", DeviceType::Laptop);
        svc.announce(ann);
        assert_eq!(svc.discover().len(), 1);
    }

    #[test]
    fn announce_replaces_duplicate() {
        let mut svc = DiscoveryService::new(None);
        let id = Uuid::new_v4();
        let mut ann1 = make_announcement("laptop-v1", DeviceType::Laptop);
        ann1.device_id = id;
        let mut ann2 = make_announcement("laptop-v2", DeviceType::Laptop);
        ann2.device_id = id;
        svc.announce(ann1);
        svc.announce(ann2);
        assert_eq!(svc.discover().len(), 1);
        assert_eq!(svc.discover()[0].name, "laptop-v2");
    }

    #[test]
    fn join_mesh_success() {
        let mesh_id = Uuid::new_v4();
        let mut svc = DiscoveryService::new(Some(mesh_id));
        let ann = make_announcement("hub", DeviceType::HomeHub);
        let dev_id = ann.device_id;
        svc.announce(ann);

        let result = svc.join_mesh(dev_id, mesh_id);
        assert!(result.accepted);
        assert_eq!(result.assigned_zone, Some("zone-c".to_string()));
    }

    #[test]
    fn join_mesh_already_in_mesh() {
        let mesh_id = Uuid::new_v4();
        let mut svc = DiscoveryService::new(Some(mesh_id));
        let ann = make_announcement("phone", DeviceType::Phone);
        let dev_id = ann.device_id;
        svc.announce(ann);

        let _ = svc.join_mesh(dev_id, mesh_id);
        let result = svc.join_mesh(dev_id, Uuid::new_v4());
        assert!(!result.accepted);
        assert!(result.reason.unwrap().contains("already"));
    }

    #[test]
    fn join_mesh_unknown_device() {
        let mut svc = DiscoveryService::new(None);
        let result = svc.join_mesh(Uuid::new_v4(), Uuid::new_v4());
        assert!(!result.accepted);
        assert!(result.reason.unwrap().contains("not found"));
    }

    #[test]
    fn discover_available_filters_joined() {
        let mesh_id = Uuid::new_v4();
        let mut svc = DiscoveryService::new(Some(mesh_id));
        let ann1 = make_announcement("laptop", DeviceType::Laptop);
        let dev1_id = ann1.device_id;
        let ann2 = make_announcement("phone", DeviceType::Phone);
        svc.announce(ann1);
        svc.announce(ann2);
        assert_eq!(svc.discover_available().len(), 2);

        let _ = svc.join_mesh(dev1_id, mesh_id);
        assert_eq!(svc.discover_available().len(), 1);
    }
}
