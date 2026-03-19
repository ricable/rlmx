//! Error types for the personal mesh bounded context.

use uuid::Uuid;

use crate::device::{DeviceType, Zone};

/// Errors produced by mesh operations.
#[derive(Debug, thiserror::Error)]
pub enum MeshError {
    #[error("device not found: {0}")]
    DeviceNotFound(Uuid),

    #[error("device already exists in mesh: {0}")]
    DeviceAlreadyExists(Uuid),

    #[error("agent not found: {0}")]
    AgentNotFound(Uuid),

    #[error("agent already assigned: {0}")]
    AgentAlreadyAssigned(Uuid),

    #[error("privacy anchor must be Zone C (Edge), got {0:?}")]
    InvalidPrivacyAnchorZone(Zone),

    #[error("privacy anchor already set to device {0}")]
    PrivacyAnchorAlreadySet(Uuid),

    #[error("mesh already has a coordinator device")]
    CoordinatorAlreadyExists,

    #[error("device {device_id} does not support zone {zone:?} (type: {device_type:?})")]
    IncompatibleZone {
        device_id: Uuid,
        zone: Zone,
        device_type: DeviceType,
    },

    #[error("sync error: {0}")]
    SyncError(String),

    #[error("discovery error: {0}")]
    DiscoveryError(String),

    #[error("failover error: {0}")]
    FailoverError(String),

    #[error("fleet manifest version mismatch: expected {expected}, got {actual}")]
    ManifestVersionMismatch { expected: u64, actual: u64 },

    #[error("max devices reached: {max}")]
    MaxDevicesReached { max: usize },

    #[error("invalid operation: {0}")]
    InvalidOperation(String),
}

pub type MeshResult<T> = Result<T, MeshError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_device_not_found() {
        let id = Uuid::new_v4();
        let err = MeshError::DeviceNotFound(id);
        assert!(err.to_string().contains(&id.to_string()));
    }

    #[test]
    fn error_display_invalid_privacy_anchor() {
        let err = MeshError::InvalidPrivacyAnchorZone(Zone::ADesktop);
        assert!(err.to_string().contains("Zone C"));
    }

    #[test]
    fn error_display_max_devices() {
        let err = MeshError::MaxDevicesReached { max: 16 };
        assert_eq!(err.to_string(), "max devices reached: 16");
    }

    #[test]
    fn mesh_result_ok() {
        let r: MeshResult<u32> = Ok(42);
        assert_eq!(r.unwrap(), 42);
    }

    #[test]
    fn mesh_result_err() {
        let r: MeshResult<u32> = Err(MeshError::SyncError("timeout".into()));
        assert!(r.is_err());
    }
}
