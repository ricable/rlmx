//! MeshDevice entity and related types for the personal mesh.
//!
//! A `MeshDevice` represents a physical device participating in the mesh,
//! with its zone assignment, capabilities, and connectivity status.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a device in the mesh.
pub type DeviceId = Uuid;

/// The zone a device is assigned to within the 5-zone topology (ADR-001/022).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Zone {
    /// Laptop / Mac / Linux desktop -- coordinator, SONA master, primary inference.
    ADesktop,
    /// Phone (iOS/Android) -- always-on agents, quick inference, UI.
    AMobile,
    /// Cloud (NUC/VM) -- burst compute, large model inference.
    BCloud,
    /// Home Hub (RPi5) -- privacy anchor, long-term storage, sentinel.
    CEdge,
    /// Browser tab -- WASM/WebGPU compute workers.
    DBrowser,
}

impl Zone {
    /// Returns the swarm zone identifier string for this zone.
    pub fn as_swarm_zone(&self) -> &'static str {
        match self {
            Zone::ADesktop => "zone-a-desktop",
            Zone::AMobile => "zone-a-mobile",
            Zone::BCloud => "zone-b",
            Zone::CEdge => "zone-c",
            Zone::DBrowser => "zone-d",
        }
    }

    /// Whether this zone can act as a coordinator.
    pub fn is_coordinator_zone(&self) -> bool {
        matches!(self, Zone::ADesktop)
    }

    /// Whether this zone can serve as a privacy anchor.
    pub fn is_privacy_anchor_zone(&self) -> bool {
        matches!(self, Zone::CEdge)
    }
}

/// Physical device type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    Laptop,
    Phone,
    HomeHub,
    CloudNode,
    Browser,
    /// Dedicated sensor appliance (e.g., Cognitum Seed on Pi Zero 2 W).
    Sensor,
}

impl DeviceType {
    /// Returns the default zone for this device type.
    pub fn default_zone(&self) -> Zone {
        match self {
            DeviceType::Laptop => Zone::ADesktop,
            DeviceType::Phone => Zone::AMobile,
            DeviceType::HomeHub => Zone::CEdge,
            DeviceType::CloudNode => Zone::BCloud,
            DeviceType::Browser => Zone::DBrowser,
            DeviceType::Sensor => Zone::CEdge,
        }
    }
}

/// Connectivity and operational status of a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceStatus {
    Online,
    Offline,
    Syncing,
    Degraded,
}

/// Hardware and runtime capabilities of a device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub has_gpu: bool,
    pub gpu_type: Option<String>,
    pub has_battery: bool,
    pub battery_pct: Option<f32>,
    pub network_type: Option<String>,
}

impl Default for DeviceCapabilities {
    fn default() -> Self {
        Self {
            cpu_cores: 4,
            memory_mb: 8192,
            has_gpu: false,
            gpu_type: None,
            has_battery: false,
            battery_pct: None,
            network_type: None,
        }
    }
}

/// A physical device participating in the personal mesh.
///
/// This is an entity (has identity and lifecycle independent of the aggregate).
/// Devices join, leave, sleep, and wake. Their zone assignment can change
/// over time as capabilities shift.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshDevice {
    pub device_id: DeviceId,
    pub name: String,
    pub device_type: DeviceType,
    pub zone: Zone,
    pub capabilities: DeviceCapabilities,
    pub status: DeviceStatus,
    pub last_seen: DateTime<Utc>,
    /// Opaque authentication token scoped to this device.
    pub token: String,
}

impl MeshDevice {
    /// Create a new device with default zone derived from its type.
    pub fn new(
        device_id: DeviceId,
        name: impl Into<String>,
        device_type: DeviceType,
        capabilities: DeviceCapabilities,
        token: impl Into<String>,
    ) -> Self {
        Self {
            device_id,
            name: name.into(),
            device_type,
            zone: device_type.default_zone(),
            capabilities,
            status: DeviceStatus::Online,
            last_seen: Utc::now(),
            token: token.into(),
        }
    }

    /// Whether this device is currently reachable.
    pub fn is_online(&self) -> bool {
        matches!(self.status, DeviceStatus::Online | DeviceStatus::Syncing)
    }

    /// Mark this device as having been seen now.
    pub fn touch(&mut self) {
        self.last_seen = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_swarm_mapping() {
        assert_eq!(Zone::ADesktop.as_swarm_zone(), "zone-a-desktop");
        assert_eq!(Zone::AMobile.as_swarm_zone(), "zone-a-mobile");
        assert_eq!(Zone::BCloud.as_swarm_zone(), "zone-b");
        assert_eq!(Zone::CEdge.as_swarm_zone(), "zone-c");
        assert_eq!(Zone::DBrowser.as_swarm_zone(), "zone-d");
    }

    #[test]
    fn zone_coordinator_and_privacy() {
        assert!(Zone::ADesktop.is_coordinator_zone());
        assert!(!Zone::AMobile.is_coordinator_zone());
        assert!(Zone::CEdge.is_privacy_anchor_zone());
        assert!(!Zone::BCloud.is_privacy_anchor_zone());
    }

    #[test]
    fn device_type_default_zones() {
        assert_eq!(DeviceType::Laptop.default_zone(), Zone::ADesktop);
        assert_eq!(DeviceType::Phone.default_zone(), Zone::AMobile);
        assert_eq!(DeviceType::HomeHub.default_zone(), Zone::CEdge);
        assert_eq!(DeviceType::CloudNode.default_zone(), Zone::BCloud);
        assert_eq!(DeviceType::Browser.default_zone(), Zone::DBrowser);
        assert_eq!(DeviceType::Sensor.default_zone(), Zone::CEdge);
    }

    #[test]
    fn mesh_device_new_defaults() {
        let dev = MeshDevice::new(
            Uuid::new_v4(),
            "my-laptop",
            DeviceType::Laptop,
            DeviceCapabilities::default(),
            "tok_abc",
        );
        assert_eq!(dev.zone, Zone::ADesktop);
        assert_eq!(dev.status, DeviceStatus::Online);
        assert!(dev.is_online());
    }

    #[test]
    fn mesh_device_offline_not_reachable() {
        let mut dev = MeshDevice::new(
            Uuid::new_v4(),
            "hub",
            DeviceType::HomeHub,
            DeviceCapabilities::default(),
            "tok_hub",
        );
        dev.status = DeviceStatus::Offline;
        assert!(!dev.is_online());
    }

    #[test]
    fn mesh_device_syncing_is_online() {
        let mut dev = MeshDevice::new(
            Uuid::new_v4(),
            "phone",
            DeviceType::Phone,
            DeviceCapabilities::default(),
            "tok_ph",
        );
        dev.status = DeviceStatus::Syncing;
        assert!(dev.is_online());
    }

    #[test]
    fn device_capabilities_default() {
        let caps = DeviceCapabilities::default();
        assert_eq!(caps.cpu_cores, 4);
        assert!(!caps.has_gpu);
        assert!(!caps.has_battery);
    }

    #[test]
    fn mesh_device_serialization_roundtrip() {
        let dev = MeshDevice::new(
            Uuid::new_v4(),
            "browser-tab",
            DeviceType::Browser,
            DeviceCapabilities {
                cpu_cores: 2,
                memory_mb: 2048,
                has_gpu: true,
                gpu_type: Some("WebGPU".into()),
                has_battery: false,
                battery_pct: None,
                network_type: Some("wifi".into()),
            },
            "tok_browser",
        );
        let json = serde_json::to_string(&dev).unwrap();
        let back: MeshDevice = serde_json::from_str(&json).unwrap();
        assert_eq!(back.device_id, dev.device_id);
        assert_eq!(back.zone, Zone::DBrowser);
        assert_eq!(back.capabilities.gpu_type, Some("WebGPU".into()));
    }
}
