use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::types::SwarmError;

/// Identifier for a virtual machine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VmId(pub String);

/// Status of a virtual machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VmStatus {
    Running,
    Pending,
    Terminated,
}

/// Specification for spawning a VM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmSpec {
    pub cpu_cores: u32,
    pub memory_gb: u32,
    pub gpu: bool,
    pub region: String,
}

/// Policy controlling cloud bursting behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudPolicy {
    pub burst_threshold: f64,
    pub max_vms: u32,
    #[serde(with = "duration_serde")]
    pub cool_down: Duration,
}

/// Trait for cloud provider implementations.
#[async_trait]
pub trait CloudProvider: Send + Sync {
    /// Spawn a new VM with the given spec.
    async fn spawn_vm(&self, spec: VmSpec) -> Result<VmId, SwarmError>;

    /// Terminate a running VM.
    async fn terminate_vm(&self, id: VmId) -> Result<(), SwarmError>;

    /// Get the status of a VM.
    async fn status(&self, id: VmId) -> Result<VmStatus, SwarmError>;
}

/// Stub cloud provider — always returns "not configured".
pub struct StubCloudProvider;

#[async_trait]
impl CloudProvider for StubCloudProvider {
    async fn spawn_vm(&self, _spec: VmSpec) -> Result<VmId, SwarmError> {
        Err(SwarmError::NotConfigured(
            "cloud provider not configured".into(),
        ))
    }

    async fn terminate_vm(&self, _id: VmId) -> Result<(), SwarmError> {
        Err(SwarmError::NotConfigured(
            "cloud provider not configured".into(),
        ))
    }

    async fn status(&self, _id: VmId) -> Result<VmStatus, SwarmError> {
        Err(SwarmError::NotConfigured(
            "cloud provider not configured".into(),
        ))
    }
}

pub(crate) mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        d.as_millis().serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let ms = u64::deserialize(d)?;
        Ok(Duration::from_millis(ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_stub_spawn_vm() {
        let provider = StubCloudProvider;
        let spec = VmSpec {
            cpu_cores: 4,
            memory_gb: 8,
            gpu: false,
            region: "us-east-1".into(),
        };
        let result = provider.spawn_vm(spec).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SwarmError::NotConfigured(msg) => assert!(msg.contains("not configured")),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[tokio::test]
    async fn test_stub_terminate_vm() {
        let provider = StubCloudProvider;
        let result = provider.terminate_vm(VmId("vm-1".into())).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stub_status() {
        let provider = StubCloudProvider;
        let result = provider.status(VmId("vm-1".into())).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_vm_spec_serialize() {
        let spec = VmSpec {
            cpu_cores: 8,
            memory_gb: 32,
            gpu: true,
            region: "eu-west-1".into(),
        };
        let json = serde_json::to_string(&spec).unwrap();
        assert!(json.contains("eu-west-1"));
    }

    #[test]
    fn test_cloud_policy_serialize() {
        let policy = CloudPolicy {
            burst_threshold: 0.8,
            max_vms: 10,
            cool_down: Duration::from_secs(60),
        };
        let json = serde_json::to_string(&policy).unwrap();
        let deserialized: CloudPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.max_vms, 10);
    }
}
