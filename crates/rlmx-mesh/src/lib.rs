//! # rlmx-mesh
//!
//! Personal mesh topology for multi-device agent coordination (ADR-022, DDD-011).
//!
//! This crate implements the PersonalMesh aggregate root, managing a single
//! user's device fleet with zone assignments, state synchronization,
//! device discovery, and graceful degradation.

pub mod device;
pub mod discovery;
pub mod error;
pub mod failover;
pub mod mesh;
pub mod sync;

pub use device::{DeviceCapabilities, DeviceId, DeviceStatus, DeviceType, MeshDevice, Zone};
pub use discovery::{DeviceAnnouncement, DiscoveryMethod, DiscoveryService, JoinResult};
pub use error::{MeshError, MeshResult};
pub use failover::{
    evaluate_degradation, DegradationLevel, FailoverPolicy, FailoverReason, MeshDegradation,
    MigrationReason,
};
pub use mesh::{FleetStatus, MeshAgent, MeshDomainEvent, MeshId, PersonalMesh, SonaSnapshot};
pub use sync::{
    SyncHealth, SyncPolicy, SyncPriority, SyncProtocol, SyncScope, SyncState, SyncTransport,
};
