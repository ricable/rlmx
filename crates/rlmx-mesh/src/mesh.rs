//! PersonalMesh aggregate root.
//!
//! The `PersonalMesh` owns the consistency boundary for a single user's
//! device fleet. All mutations to device membership, zone assignments,
//! agent placement, and sync state flow through the mesh.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use rlmx_kernel::ProcessId;
use rlmx_swarm::sandbox::FleetManifest;

use crate::device::{DeviceCapabilities, DeviceId, DeviceStatus, DeviceType, MeshDevice, Zone};
use crate::error::{MeshError, MeshResult};
use crate::failover::MigrationReason;
use crate::sync::{SyncState, SyncTransport};

/// Unique identifier for a mesh instance.
pub type MeshId = Uuid;

/// An agent instance running on a specific device within the mesh.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshAgent {
    pub agent_id: Uuid,
    pub agent_type: String,
    pub device_id: DeviceId,
    pub process_id: Option<ProcessId>,
    pub sona_snapshot: SonaSnapshot,
}

/// Snapshot of an agent's SONA learning state for sync purposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SonaSnapshot {
    pub last_sync: DateTime<Utc>,
    pub pattern_count: u64,
}

impl Default for SonaSnapshot {
    fn default() -> Self {
        Self {
            last_sync: Utc::now(),
            pattern_count: 0,
        }
    }
}

/// Domain events emitted by mesh operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeshDomainEvent {
    DeviceJoined {
        mesh_id: MeshId,
        device_id: DeviceId,
        zone: Zone,
    },
    DeviceLeft {
        mesh_id: MeshId,
        device_id: DeviceId,
        graceful: bool,
        pending_ops: u64,
    },
    MeshReconfigured {
        mesh_id: MeshId,
        manifest_version: u64,
        changes: Vec<String>,
    },
    SyncCompleted {
        source_device: DeviceId,
        target_device: DeviceId,
        ops_synced: u64,
    },
    ZoneFailover {
        mesh_id: MeshId,
        from_zone: Zone,
        to_zone: Zone,
        promoted_device: DeviceId,
    },
    PrivacyAnchorRestored {
        mesh_id: MeshId,
        anchor_device: DeviceId,
        queued_syncs: u64,
    },
    AgentMigrated {
        agent_id: Uuid,
        from_device: DeviceId,
        to_device: DeviceId,
        reason: MigrationReason,
    },
}

/// Maximum number of devices in a single personal mesh (ADR-022).
const MAX_DEVICES: usize = 10;

/// The PersonalMesh aggregate root.
///
/// Owns the consistency boundary for a single user's device fleet.
/// All mutations to device membership, zone assignments, agent placement,
/// and sync state flow through this aggregate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalMesh {
    pub id: MeshId,
    pub devices: Vec<MeshDevice>,
    pub agents: Vec<MeshAgent>,
    pub sync_states: Vec<SyncState>,
    pub fleet_manifest: Option<FleetManifest>,
    pub privacy_anchor: Option<DeviceId>,
    pub created_at: DateTime<Utc>,
    pub last_reconfigured: DateTime<Utc>,
    /// Pending domain events to be dispatched.
    #[serde(skip)]
    pending_events: Vec<MeshDomainEvent>,
}

impl PersonalMesh {
    /// Create a new empty mesh.
    pub fn new(id: MeshId) -> Self {
        let now = Utc::now();
        Self {
            id,
            devices: Vec::new(),
            agents: Vec::new(),
            sync_states: Vec::new(),
            fleet_manifest: None,
            privacy_anchor: None,
            created_at: now,
            last_reconfigured: now,
            pending_events: Vec::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Device management
    // -----------------------------------------------------------------------

    /// Add a device to the mesh with its default zone.
    pub fn add_device(
        &mut self,
        device_id: DeviceId,
        name: impl Into<String>,
        device_type: DeviceType,
        capabilities: DeviceCapabilities,
        token: impl Into<String>,
    ) -> MeshResult<()> {
        if self.devices.len() >= MAX_DEVICES {
            return Err(MeshError::MaxDevicesReached { max: MAX_DEVICES });
        }
        if self.devices.iter().any(|d| d.device_id == device_id) {
            return Err(MeshError::DeviceAlreadyExists(device_id));
        }

        let device = MeshDevice::new(device_id, name, device_type, capabilities, token);
        let zone = device.zone;

        // Create sync states between the new device and all existing devices.
        for existing in &self.devices {
            let transport = select_transport(existing.zone, zone);
            self.sync_states
                .push(SyncState::new(existing.device_id, device_id, transport));
        }

        tracing::info!(
            mesh_id = %self.id,
            device_id = %device_id,
            zone = ?zone,
            "Device added to mesh"
        );

        self.devices.push(device);
        self.last_reconfigured = Utc::now();

        self.pending_events.push(MeshDomainEvent::DeviceJoined {
            mesh_id: self.id,
            device_id,
            zone,
        });

        Ok(())
    }

    /// Remove a device from the mesh.
    pub fn remove_device(&mut self, device_id: DeviceId, graceful: bool) -> MeshResult<()> {
        if !self.devices.iter().any(|d| d.device_id == device_id) {
            return Err(MeshError::DeviceNotFound(device_id));
        }

        // Calculate pending ops for the departing device.
        let pending_ops: u64 = self
            .sync_states
            .iter()
            .filter(|s| s.source_device == device_id || s.target_device == device_id)
            .map(|s| s.pending_ops)
            .sum();

        // Remove sync states involving this device.
        self.sync_states
            .retain(|s| s.source_device != device_id && s.target_device != device_id);

        // Clear privacy anchor if this was the anchor device.
        if self.privacy_anchor == Some(device_id) {
            self.privacy_anchor = None;
            tracing::warn!(
                mesh_id = %self.id,
                device_id = %device_id,
                "Privacy anchor removed with departing device"
            );
        }

        self.devices.retain(|d| d.device_id != device_id);
        self.last_reconfigured = Utc::now();

        tracing::info!(
            mesh_id = %self.id,
            device_id = %device_id,
            graceful = graceful,
            pending_ops = pending_ops,
            "Device removed from mesh"
        );

        self.pending_events.push(MeshDomainEvent::DeviceLeft {
            mesh_id: self.id,
            device_id,
            graceful,
            pending_ops,
        });

        Ok(())
    }

    /// Reassign a device to a different zone.
    pub fn assign_zone(&mut self, device_id: DeviceId, zone: Zone) -> MeshResult<()> {
        let device = self
            .devices
            .iter_mut()
            .find(|d| d.device_id == device_id)
            .ok_or(MeshError::DeviceNotFound(device_id))?;

        let old_zone = device.zone;
        device.zone = zone;
        self.last_reconfigured = Utc::now();

        tracing::info!(
            mesh_id = %self.id,
            device_id = %device_id,
            old_zone = ?old_zone,
            new_zone = ?zone,
            "Device zone reassigned"
        );

        self.pending_events.push(MeshDomainEvent::MeshReconfigured {
            mesh_id: self.id,
            manifest_version: 0,
            changes: vec![format!(
                "zone reassignment: {device_id} from {old_zone:?} to {zone:?}"
            )],
        });

        Ok(())
    }

    /// Set a device as the privacy anchor (must be Zone C / Edge).
    pub fn set_privacy_anchor(&mut self, device_id: DeviceId) -> MeshResult<()> {
        let device = self
            .devices
            .iter()
            .find(|d| d.device_id == device_id)
            .ok_or(MeshError::DeviceNotFound(device_id))?;

        if !device.zone.is_privacy_anchor_zone() {
            return Err(MeshError::InvalidPrivacyAnchorZone(device.zone));
        }

        self.privacy_anchor = Some(device_id);
        self.last_reconfigured = Utc::now();

        tracing::info!(
            mesh_id = %self.id,
            device_id = %device_id,
            "Privacy anchor set"
        );

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Agent management
    // -----------------------------------------------------------------------

    /// Place an agent on a specific device.
    pub fn place_agent(
        &mut self,
        agent_id: Uuid,
        agent_type: impl Into<String>,
        device_id: DeviceId,
    ) -> MeshResult<()> {
        if !self.devices.iter().any(|d| d.device_id == device_id) {
            return Err(MeshError::DeviceNotFound(device_id));
        }
        if self.agents.iter().any(|a| a.agent_id == agent_id) {
            return Err(MeshError::AgentAlreadyAssigned(agent_id));
        }

        self.agents.push(MeshAgent {
            agent_id,
            agent_type: agent_type.into(),
            device_id,
            process_id: None,
            sona_snapshot: SonaSnapshot::default(),
        });

        tracing::info!(
            mesh_id = %self.id,
            agent_id = %agent_id,
            device_id = %device_id,
            "Agent placed on device"
        );

        Ok(())
    }

    /// Migrate an agent from one device to another.
    pub fn migrate_agent(
        &mut self,
        agent_id: Uuid,
        target_device: DeviceId,
        reason: MigrationReason,
    ) -> MeshResult<()> {
        if !self.devices.iter().any(|d| d.device_id == target_device) {
            return Err(MeshError::DeviceNotFound(target_device));
        }

        let agent = self
            .agents
            .iter_mut()
            .find(|a| a.agent_id == agent_id)
            .ok_or(MeshError::AgentNotFound(agent_id))?;

        let from_device = agent.device_id;
        agent.device_id = target_device;

        tracing::info!(
            mesh_id = %self.id,
            agent_id = %agent_id,
            from_device = %from_device,
            to_device = %target_device,
            reason = ?reason,
            "Agent migrated"
        );

        self.pending_events.push(MeshDomainEvent::AgentMigrated {
            agent_id,
            from_device,
            to_device: target_device,
            reason,
        });

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Get all agents running on a specific device.
    pub fn agents_on_device(&self, device_id: DeviceId) -> Vec<&MeshAgent> {
        self.agents
            .iter()
            .filter(|a| a.device_id == device_id)
            .collect()
    }

    /// Get all devices in a specific zone.
    pub fn devices_in_zone(&self, zone: Zone) -> Vec<&MeshDevice> {
        self.devices.iter().filter(|d| d.zone == zone).collect()
    }

    /// Get online devices only.
    pub fn online_devices(&self) -> Vec<&MeshDevice> {
        self.devices.iter().filter(|d| d.is_online()).collect()
    }

    /// Get the coordinator device (Zone A-Desktop), if present and online.
    pub fn coordinator(&self) -> Option<&MeshDevice> {
        self.devices
            .iter()
            .find(|d| d.zone.is_coordinator_zone() && d.is_online())
    }

    /// Get the privacy anchor device, if set.
    pub fn privacy_anchor_device(&self) -> Option<&MeshDevice> {
        self.privacy_anchor
            .and_then(|id| self.devices.iter().find(|d| d.device_id == id))
    }

    /// Get fleet status summary.
    pub fn get_fleet_status(&self) -> FleetStatus {
        let online = self.devices.iter().filter(|d| d.is_online()).count();
        let total = self.devices.len();
        let total_agents = self.agents.len();
        let pending_sync: u64 = self.sync_states.iter().map(|s| s.pending_ops).sum();
        let has_anchor = self.privacy_anchor.is_some();

        FleetStatus {
            mesh_id: self.id,
            total_devices: total,
            online_devices: online,
            total_agents,
            pending_sync_ops: pending_sync,
            has_privacy_anchor: has_anchor,
            has_coordinator: self.coordinator().is_some(),
        }
    }

    /// Drain and return pending domain events.
    pub fn take_events(&mut self) -> Vec<MeshDomainEvent> {
        std::mem::take(&mut self.pending_events)
    }

    /// Update a device's status.
    pub fn update_device_status(
        &mut self,
        device_id: DeviceId,
        status: DeviceStatus,
    ) -> MeshResult<()> {
        let device = self
            .devices
            .iter_mut()
            .find(|d| d.device_id == device_id)
            .ok_or(MeshError::DeviceNotFound(device_id))?;
        device.status = status;
        device.touch();
        Ok(())
    }
}

/// Summary of mesh fleet status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetStatus {
    pub mesh_id: MeshId,
    pub total_devices: usize,
    pub online_devices: usize,
    pub total_agents: usize,
    pub pending_sync_ops: u64,
    pub has_privacy_anchor: bool,
    pub has_coordinator: bool,
}

/// Select the appropriate transport for a device pair based on their zones.
fn select_transport(zone_a: Zone, zone_b: Zone) -> SyncTransport {
    match (zone_a, zone_b) {
        // LAN devices use QUIC for sub-millisecond sync.
        (Zone::ADesktop, Zone::CEdge) | (Zone::CEdge, Zone::ADesktop) => SyncTransport::Quic,
        // Browser uses BroadcastChannel.
        (Zone::DBrowser, _) | (_, Zone::DBrowser) => SyncTransport::BroadcastChannel,
        // Phone uses WebSocket via cloud relay.
        (Zone::AMobile, _) | (_, Zone::AMobile) => SyncTransport::WebSocket,
        // Cloud uses HTTP.
        (Zone::BCloud, _) | (_, Zone::BCloud) => SyncTransport::Http,
        // Default to WebSocket.
        _ => SyncTransport::WebSocket,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mesh() -> PersonalMesh {
        PersonalMesh::new(Uuid::new_v4())
    }

    fn add_laptop(mesh: &mut PersonalMesh) -> DeviceId {
        let id = Uuid::new_v4();
        mesh.add_device(
            id,
            "laptop",
            DeviceType::Laptop,
            DeviceCapabilities::default(),
            "tok",
        )
        .unwrap();
        id
    }

    fn add_hub(mesh: &mut PersonalMesh) -> DeviceId {
        let id = Uuid::new_v4();
        mesh.add_device(
            id,
            "hub",
            DeviceType::HomeHub,
            DeviceCapabilities::default(),
            "tok",
        )
        .unwrap();
        id
    }

    #[test]
    fn add_device_success() {
        let mut mesh = create_mesh();
        let id = add_laptop(&mut mesh);
        assert_eq!(mesh.devices.len(), 1);
        assert_eq!(mesh.devices[0].device_id, id);
        assert_eq!(mesh.devices[0].zone, Zone::ADesktop);
    }

    #[test]
    fn add_device_creates_sync_states() {
        let mut mesh = create_mesh();
        add_laptop(&mut mesh);
        add_hub(&mut mesh);
        // One sync state for the pair.
        assert_eq!(mesh.sync_states.len(), 1);
    }

    #[test]
    fn add_device_duplicate_rejected() {
        let mut mesh = create_mesh();
        let id = Uuid::new_v4();
        mesh.add_device(
            id,
            "a",
            DeviceType::Laptop,
            DeviceCapabilities::default(),
            "t",
        )
        .unwrap();
        let err = mesh
            .add_device(
                id,
                "b",
                DeviceType::Phone,
                DeviceCapabilities::default(),
                "t",
            )
            .unwrap_err();
        assert!(matches!(err, MeshError::DeviceAlreadyExists(_)));
    }

    #[test]
    fn add_device_max_limit() {
        let mut mesh = create_mesh();
        for i in 0..MAX_DEVICES {
            mesh.add_device(
                Uuid::new_v4(),
                format!("dev-{i}"),
                DeviceType::Browser,
                DeviceCapabilities::default(),
                "t",
            )
            .unwrap();
        }
        let err = mesh
            .add_device(
                Uuid::new_v4(),
                "one-too-many",
                DeviceType::Browser,
                DeviceCapabilities::default(),
                "t",
            )
            .unwrap_err();
        assert!(matches!(err, MeshError::MaxDevicesReached { .. }));
    }

    #[test]
    fn remove_device_success() {
        let mut mesh = create_mesh();
        let id = add_laptop(&mut mesh);
        mesh.remove_device(id, true).unwrap();
        assert!(mesh.devices.is_empty());
    }

    #[test]
    fn remove_device_clears_sync_states() {
        let mut mesh = create_mesh();
        let id1 = add_laptop(&mut mesh);
        let _id2 = add_hub(&mut mesh);
        assert_eq!(mesh.sync_states.len(), 1);
        mesh.remove_device(id1, true).unwrap();
        assert!(mesh.sync_states.is_empty());
    }

    #[test]
    fn remove_device_clears_privacy_anchor() {
        let mut mesh = create_mesh();
        let hub_id = add_hub(&mut mesh);
        mesh.set_privacy_anchor(hub_id).unwrap();
        assert!(mesh.privacy_anchor.is_some());
        mesh.remove_device(hub_id, false).unwrap();
        assert!(mesh.privacy_anchor.is_none());
    }

    #[test]
    fn remove_unknown_device_errors() {
        let mut mesh = create_mesh();
        let err = mesh.remove_device(Uuid::new_v4(), true).unwrap_err();
        assert!(matches!(err, MeshError::DeviceNotFound(_)));
    }

    #[test]
    fn assign_zone() {
        let mut mesh = create_mesh();
        let id = add_laptop(&mut mesh);
        mesh.assign_zone(id, Zone::BCloud).unwrap();
        assert_eq!(mesh.devices[0].zone, Zone::BCloud);
    }

    #[test]
    fn set_privacy_anchor_requires_edge_zone() {
        let mut mesh = create_mesh();
        let laptop_id = add_laptop(&mut mesh);
        let err = mesh.set_privacy_anchor(laptop_id).unwrap_err();
        assert!(matches!(err, MeshError::InvalidPrivacyAnchorZone(_)));
    }

    #[test]
    fn set_privacy_anchor_success() {
        let mut mesh = create_mesh();
        let hub_id = add_hub(&mut mesh);
        mesh.set_privacy_anchor(hub_id).unwrap();
        assert_eq!(mesh.privacy_anchor, Some(hub_id));
    }

    #[test]
    fn place_and_query_agents() {
        let mut mesh = create_mesh();
        let dev_id = add_laptop(&mut mesh);
        let agent_id = Uuid::new_v4();
        mesh.place_agent(agent_id, "Worker", dev_id).unwrap();
        let agents = mesh.agents_on_device(dev_id);
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].agent_type, "Worker");
    }

    #[test]
    fn place_agent_on_unknown_device_fails() {
        let mut mesh = create_mesh();
        let err = mesh
            .place_agent(Uuid::new_v4(), "Worker", Uuid::new_v4())
            .unwrap_err();
        assert!(matches!(err, MeshError::DeviceNotFound(_)));
    }

    #[test]
    fn place_duplicate_agent_fails() {
        let mut mesh = create_mesh();
        let dev_id = add_laptop(&mut mesh);
        let agent_id = Uuid::new_v4();
        mesh.place_agent(agent_id, "Worker", dev_id).unwrap();
        let err = mesh.place_agent(agent_id, "Worker", dev_id).unwrap_err();
        assert!(matches!(err, MeshError::AgentAlreadyAssigned(_)));
    }

    #[test]
    fn migrate_agent_success() {
        let mut mesh = create_mesh();
        let dev1 = add_laptop(&mut mesh);
        let dev2 = add_hub(&mut mesh);
        let agent_id = Uuid::new_v4();
        mesh.place_agent(agent_id, "Worker", dev1).unwrap();
        mesh.migrate_agent(agent_id, dev2, MigrationReason::LoadBalancing)
            .unwrap();
        assert!(mesh.agents_on_device(dev1).is_empty());
        assert_eq!(mesh.agents_on_device(dev2).len(), 1);
    }

    #[test]
    fn fleet_status() {
        let mut mesh = create_mesh();
        let dev1 = add_laptop(&mut mesh);
        let _dev2 = add_hub(&mut mesh);
        mesh.place_agent(Uuid::new_v4(), "Worker", dev1).unwrap();

        let status = mesh.get_fleet_status();
        assert_eq!(status.total_devices, 2);
        assert_eq!(status.online_devices, 2);
        assert_eq!(status.total_agents, 1);
        assert!(!status.has_privacy_anchor);
        assert!(status.has_coordinator);
    }

    #[test]
    fn devices_in_zone_query() {
        let mut mesh = create_mesh();
        add_laptop(&mut mesh);
        add_hub(&mut mesh);
        assert_eq!(mesh.devices_in_zone(Zone::ADesktop).len(), 1);
        assert_eq!(mesh.devices_in_zone(Zone::CEdge).len(), 1);
        assert_eq!(mesh.devices_in_zone(Zone::BCloud).len(), 0);
    }

    #[test]
    fn take_events_drains() {
        let mut mesh = create_mesh();
        add_laptop(&mut mesh);
        let events = mesh.take_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], MeshDomainEvent::DeviceJoined { .. }));
        // Second take returns empty.
        assert!(mesh.take_events().is_empty());
    }

    #[test]
    fn select_transport_quic_for_lan() {
        assert_eq!(
            select_transport(Zone::ADesktop, Zone::CEdge),
            SyncTransport::Quic
        );
    }

    #[test]
    fn select_transport_ws_for_phone() {
        assert_eq!(
            select_transport(Zone::AMobile, Zone::BCloud),
            SyncTransport::WebSocket
        );
    }

    #[test]
    fn select_transport_broadcast_for_browser() {
        assert_eq!(
            select_transport(Zone::DBrowser, Zone::ADesktop),
            SyncTransport::BroadcastChannel
        );
    }

    #[test]
    fn update_device_status() {
        let mut mesh = create_mesh();
        let id = add_laptop(&mut mesh);
        mesh.update_device_status(id, DeviceStatus::Degraded)
            .unwrap();
        assert_eq!(mesh.devices[0].status, DeviceStatus::Degraded);
    }

    #[test]
    fn coordinator_returns_desktop() {
        let mut mesh = create_mesh();
        let id = add_laptop(&mut mesh);
        assert_eq!(mesh.coordinator().unwrap().device_id, id);
    }

    #[test]
    fn coordinator_none_when_offline() {
        let mut mesh = create_mesh();
        let id = add_laptop(&mut mesh);
        mesh.update_device_status(id, DeviceStatus::Offline)
            .unwrap();
        assert!(mesh.coordinator().is_none());
    }

    #[test]
    fn privacy_anchor_device_query() {
        let mut mesh = create_mesh();
        let hub_id = add_hub(&mut mesh);
        mesh.set_privacy_anchor(hub_id).unwrap();
        assert_eq!(mesh.privacy_anchor_device().unwrap().device_id, hub_id);
    }

    #[test]
    fn mesh_serialization_roundtrip() {
        let mut mesh = create_mesh();
        add_laptop(&mut mesh);
        let hub_id = add_hub(&mut mesh);
        mesh.set_privacy_anchor(hub_id).unwrap();
        mesh.place_agent(Uuid::new_v4(), "Worker", hub_id).unwrap();

        let json = serde_json::to_string(&mesh).unwrap();
        let back: PersonalMesh = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, mesh.id);
        assert_eq!(back.devices.len(), 2);
        assert_eq!(back.agents.len(), 1);
        assert_eq!(back.privacy_anchor, Some(hub_id));
    }
}
