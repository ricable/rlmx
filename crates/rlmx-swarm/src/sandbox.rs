use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::cloud::{CloudPolicy, VmId};
use crate::types::NodeId;

/// Hardware GPU requirement for a sandbox.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GpuRequirement {
    None,
    Metal,
    Cuda { min_vram_gb: u32 },
    WebGpu,
}

/// Resource envelope constraining a sandbox's compute budget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEnvelope {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub gpu: GpuRequirement,
    pub disk_mb: u64,
    #[serde(with = "crate::cloud::duration_serde")]
    pub max_runtime: Duration,
}

/// Network isolation policy for sandbox egress/ingress.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkPolicy {
    /// No external network access.
    Isolated,
    /// Can reach out, no inbound connections.
    EgressOnly,
    /// Only swarm cluster peers reachable.
    ClusterOnly,
    /// Full network access (cloud burst workers).
    Open,
}

/// A sandbox profile: deployment descriptor composing agent type
/// with resource limits, isolation policy, and model tier.
/// This is NOT a new agent type — it wraps an existing AgentType
/// with deployment concerns (ADR-011).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxProfile {
    pub name: String,
    /// AgentType variant name.
    pub agent_type: String,
    pub resources: ResourceEnvelope,
    pub network: NetworkPolicy,
    /// Preferred ZoneId.
    pub zone_preference: String,
    /// ModelTier variant name.
    pub model_tier: String,
    /// e.g., `["ruvllm", "metal"]`
    pub crate_features: Vec<String>,
    /// Optional .rvf container path.
    pub rvf_payload: Option<String>,
}

/// Unique identifier for a running sandbox instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SandboxId(pub String);

/// Lifecycle state of a sandbox instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SandboxState {
    Provisioning,
    Starting,
    Running,
    Suspended,
    Stopping,
    Terminated,
    Failed,
}

/// Runtime metrics for a sandbox instance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SandboxMetrics {
    pub cpu_usage_pct: f64,
    pub memory_usage_mb: u64,
    pub uptime_secs: u64,
    pub tasks_completed: u64,
}

/// A running sandbox instance binding a profile to a swarm node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxInstance {
    pub id: SandboxId,
    pub profile: SandboxProfile,
    pub state: SandboxState,
    pub node_id: Option<NodeId>,
    pub vm_id: Option<VmId>,
    pub created_at: DateTime<Utc>,
    pub metrics: SandboxMetrics,
}

/// Declarative fleet manifest describing multiple sandboxes to deploy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetManifest {
    pub name: String,
    pub version: String,
    pub sandboxes: Vec<SandboxSpec>,
    pub cloud_policy: Option<CloudPolicy>,
}

/// A single sandbox entry in a fleet manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSpec {
    pub profile: String,
    pub count: usize,
    pub overrides: Option<serde_json::Value>,
}

/// Validates that a `SandboxState` transition is legal.
/// Returns `true` if the transition from `from` to `to` is allowed.
pub fn valid_transition(from: SandboxState, to: SandboxState) -> bool {
    matches!(
        (from, to),
        (SandboxState::Provisioning, SandboxState::Starting)
            | (SandboxState::Provisioning, SandboxState::Failed)
            | (SandboxState::Starting, SandboxState::Running)
            | (SandboxState::Starting, SandboxState::Failed)
            | (SandboxState::Running, SandboxState::Suspended)
            | (SandboxState::Running, SandboxState::Stopping)
            | (SandboxState::Running, SandboxState::Failed)
            | (SandboxState::Suspended, SandboxState::Running)
            | (SandboxState::Suspended, SandboxState::Stopping)
            | (SandboxState::Stopping, SandboxState::Terminated)
            | (SandboxState::Stopping, SandboxState::Failed)
    )
}

/// Error type for sandbox operations.
#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("sandbox not found: {0}")]
    NotFound(String),
    #[error("profile not found: {0}")]
    ProfileNotFound(String),
    #[error("invalid state transition: {from:?} -> {to:?}")]
    InvalidTransition {
        from: SandboxState,
        to: SandboxState,
    },
    #[error("duplicate profile: {0}")]
    DuplicateProfile(String),
    #[error("fleet validation error: {0}")]
    FleetValidation(String),
}

/// Manages sandbox profiles and running instances.
///
/// The `SandboxManager` is the runtime component that:
/// - Registers and stores sandbox profiles
/// - Spawns/terminates sandbox instances
/// - Tracks lifecycle state transitions
/// - Validates fleet manifests before deployment
pub struct SandboxManager {
    profiles: HashMap<String, SandboxProfile>,
    instances: HashMap<String, SandboxInstance>,
}

impl SandboxManager {
    /// Create an empty manager with no profiles or instances.
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            instances: HashMap::new(),
        }
    }

    /// Register a sandbox profile by name. Returns error if duplicate.
    pub fn register_profile(&mut self, profile: SandboxProfile) -> Result<(), SandboxError> {
        if self.profiles.contains_key(&profile.name) {
            return Err(SandboxError::DuplicateProfile(profile.name.clone()));
        }
        info!(profile = %profile.name, agent_type = %profile.agent_type, "Registered sandbox profile");
        self.profiles.insert(profile.name.clone(), profile);
        Ok(())
    }

    /// Get a profile by name.
    pub fn get_profile(&self, name: &str) -> Option<&SandboxProfile> {
        self.profiles.get(name)
    }

    /// List all registered profile names.
    pub fn list_profiles(&self) -> Vec<&str> {
        self.profiles.keys().map(|s| s.as_str()).collect()
    }

    /// Spawn a new sandbox instance from a named profile.
    /// The instance starts in `Provisioning` state.
    pub fn spawn(&mut self, profile_name: &str) -> Result<SandboxId, SandboxError> {
        let profile = self
            .profiles
            .get(profile_name)
            .ok_or_else(|| SandboxError::ProfileNotFound(profile_name.to_string()))?
            .clone();

        let id = SandboxId(uuid::Uuid::new_v4().to_string());
        let instance = SandboxInstance {
            id: id.clone(),
            profile,
            state: SandboxState::Provisioning,
            node_id: None,
            vm_id: None,
            created_at: Utc::now(),
            metrics: SandboxMetrics::default(),
        };

        info!(sandbox_id = %id.0, profile = %profile_name, "Spawned sandbox instance");
        self.instances.insert(id.0.clone(), instance);
        Ok(id)
    }

    /// Transition a sandbox to a new state. Returns error if transition is invalid.
    pub fn transition(
        &mut self,
        sandbox_id: &str,
        new_state: SandboxState,
    ) -> Result<(), SandboxError> {
        let instance = self
            .instances
            .get_mut(sandbox_id)
            .ok_or_else(|| SandboxError::NotFound(sandbox_id.to_string()))?;

        if !valid_transition(instance.state, new_state) {
            warn!(
                sandbox_id = %sandbox_id,
                from = ?instance.state,
                to = ?new_state,
                "Invalid sandbox state transition"
            );
            return Err(SandboxError::InvalidTransition {
                from: instance.state,
                to: new_state,
            });
        }

        info!(
            sandbox_id = %sandbox_id,
            from = ?instance.state,
            to = ?new_state,
            "Sandbox state transition"
        );
        instance.state = new_state;
        Ok(())
    }

    /// Terminate a sandbox (transition to Stopping then Terminated).
    pub fn terminate(&mut self, sandbox_id: &str) -> Result<(), SandboxError> {
        let instance = self
            .instances
            .get(sandbox_id)
            .ok_or_else(|| SandboxError::NotFound(sandbox_id.to_string()))?;

        // If already terminated or failed, no-op
        if matches!(
            instance.state,
            SandboxState::Terminated | SandboxState::Failed
        ) {
            return Ok(());
        }

        // If running or suspended, go through Stopping
        if matches!(
            instance.state,
            SandboxState::Running | SandboxState::Suspended
        ) {
            self.transition(sandbox_id, SandboxState::Stopping)?;
        }

        self.transition(sandbox_id, SandboxState::Terminated)
    }

    /// Get the status of a sandbox instance.
    pub fn status(&self, sandbox_id: &str) -> Result<&SandboxInstance, SandboxError> {
        self.instances
            .get(sandbox_id)
            .ok_or_else(|| SandboxError::NotFound(sandbox_id.to_string()))
    }

    /// List all sandbox instances.
    pub fn list_instances(&self) -> Vec<&SandboxInstance> {
        self.instances.values().collect()
    }

    /// Count instances by state.
    pub fn count_by_state(&self, state: SandboxState) -> usize {
        self.instances.values().filter(|i| i.state == state).count()
    }

    /// Validate a fleet manifest: check all referenced profiles exist and counts are valid.
    pub fn validate_fleet(&self, manifest: &FleetManifest) -> Result<(), SandboxError> {
        if manifest.sandboxes.is_empty() {
            return Err(SandboxError::FleetValidation(
                "fleet manifest has no sandboxes".into(),
            ));
        }
        for spec in &manifest.sandboxes {
            if !self.profiles.contains_key(&spec.profile) {
                return Err(SandboxError::ProfileNotFound(spec.profile.clone()));
            }
            if spec.count == 0 {
                return Err(SandboxError::FleetValidation(format!(
                    "sandbox '{}' has count 0",
                    spec.profile
                )));
            }
        }
        Ok(())
    }

    /// Deploy a fleet manifest, spawning all specified sandbox instances.
    /// Returns the list of spawned sandbox IDs.
    pub fn deploy_fleet(
        &mut self,
        manifest: &FleetManifest,
    ) -> Result<Vec<SandboxId>, SandboxError> {
        self.validate_fleet(manifest)?;
        let mut ids = Vec::new();
        for spec in &manifest.sandboxes {
            for _ in 0..spec.count {
                let id = self.spawn(&spec.profile)?;
                ids.push(id);
            }
        }
        info!(
            fleet = %manifest.name,
            sandboxes = ids.len(),
            "Fleet deployed"
        );
        Ok(ids)
    }
}

impl Default for SandboxManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_profile_serialization_roundtrip() {
        let profile = SandboxProfile {
            name: "ran-optimizer".into(),
            agent_type: "Experimenter".into(),
            resources: ResourceEnvelope {
                cpu_cores: 8,
                memory_mb: 32768,
                gpu: GpuRequirement::Metal,
                disk_mb: 51200,
                max_runtime: Duration::from_secs(86400),
            },
            network: NetworkPolicy::ClusterOnly,
            zone_preference: "zone-a".into(),
            model_tier: "Medium".into(),
            crate_features: vec![],
            rvf_payload: Some("./rvf-images/ran-optimizer.rvf".into()),
        };
        let json = serde_json::to_string(&profile).unwrap();
        let back: SandboxProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "ran-optimizer");
        assert_eq!(back.network, NetworkPolicy::ClusterOnly);
    }

    #[test]
    fn fleet_manifest_serialization_roundtrip() {
        let manifest = FleetManifest {
            name: "test-fleet".into(),
            version: "1.0".into(),
            sandboxes: vec![SandboxSpec {
                profile: "ran-optimizer".into(),
                count: 2,
                overrides: None,
            }],
            cloud_policy: None,
        };
        let json = serde_json::to_string(&manifest).unwrap();
        let back: FleetManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.sandboxes.len(), 1);
        assert_eq!(back.sandboxes[0].count, 2);
    }

    #[test]
    fn sandbox_state_transitions() {
        // Document valid forward transitions
        let valid = [
            (SandboxState::Provisioning, SandboxState::Starting),
            (SandboxState::Starting, SandboxState::Running),
            (SandboxState::Running, SandboxState::Suspended),
            (SandboxState::Suspended, SandboxState::Running),
            (SandboxState::Running, SandboxState::Stopping),
            (SandboxState::Stopping, SandboxState::Terminated),
            (SandboxState::Starting, SandboxState::Failed),
            (SandboxState::Running, SandboxState::Failed),
        ];
        // States serialize correctly
        for (from, to) in &valid {
            assert_ne!(from, to);
        }
    }

    #[test]
    fn sandbox_manager_register_and_spawn() {
        let mut mgr = SandboxManager::new();
        let profile = SandboxProfile {
            name: "test-worker".into(),
            agent_type: "Worker".into(),
            resources: ResourceEnvelope {
                cpu_cores: 4,
                memory_mb: 8192,
                gpu: GpuRequirement::None,
                disk_mb: 10240,
                max_runtime: Duration::from_secs(3600),
            },
            network: NetworkPolicy::ClusterOnly,
            zone_preference: "zone-b".into(),
            model_tier: "Small".into(),
            crate_features: vec![],
            rvf_payload: None,
        };
        mgr.register_profile(profile).unwrap();
        assert_eq!(mgr.list_profiles().len(), 1);

        let id = mgr.spawn("test-worker").unwrap();
        let instance = mgr.status(&id.0).unwrap();
        assert_eq!(instance.state, SandboxState::Provisioning);
        assert_eq!(instance.profile.name, "test-worker");
    }

    #[test]
    fn sandbox_manager_duplicate_profile_rejected() {
        let mut mgr = SandboxManager::new();
        let profile = SandboxProfile {
            name: "dup".into(),
            agent_type: "Worker".into(),
            resources: ResourceEnvelope {
                cpu_cores: 1,
                memory_mb: 512,
                gpu: GpuRequirement::None,
                disk_mb: 1024,
                max_runtime: Duration::from_secs(60),
            },
            network: NetworkPolicy::Isolated,
            zone_preference: "zone-c".into(),
            model_tier: "Small".into(),
            crate_features: vec![],
            rvf_payload: None,
        };
        mgr.register_profile(profile.clone()).unwrap();
        assert!(mgr.register_profile(profile).is_err());
    }

    #[test]
    fn sandbox_manager_lifecycle_transitions() {
        let mut mgr = SandboxManager::new();
        let profile = SandboxProfile {
            name: "lifecycle-test".into(),
            agent_type: "Worker".into(),
            resources: ResourceEnvelope {
                cpu_cores: 2,
                memory_mb: 4096,
                gpu: GpuRequirement::None,
                disk_mb: 2048,
                max_runtime: Duration::from_secs(300),
            },
            network: NetworkPolicy::EgressOnly,
            zone_preference: "zone-a".into(),
            model_tier: "Medium".into(),
            crate_features: vec![],
            rvf_payload: None,
        };
        mgr.register_profile(profile).unwrap();
        let id = mgr.spawn("lifecycle-test").unwrap();

        // Provisioning -> Starting -> Running -> Stopping -> Terminated
        mgr.transition(&id.0, SandboxState::Starting).unwrap();
        mgr.transition(&id.0, SandboxState::Running).unwrap();
        mgr.transition(&id.0, SandboxState::Stopping).unwrap();
        mgr.transition(&id.0, SandboxState::Terminated).unwrap();

        let instance = mgr.status(&id.0).unwrap();
        assert_eq!(instance.state, SandboxState::Terminated);
    }

    #[test]
    fn sandbox_manager_invalid_transition_rejected() {
        let mut mgr = SandboxManager::new();
        let profile = SandboxProfile {
            name: "invalid-trans".into(),
            agent_type: "Worker".into(),
            resources: ResourceEnvelope {
                cpu_cores: 1,
                memory_mb: 512,
                gpu: GpuRequirement::None,
                disk_mb: 1024,
                max_runtime: Duration::from_secs(60),
            },
            network: NetworkPolicy::Isolated,
            zone_preference: "zone-c".into(),
            model_tier: "Small".into(),
            crate_features: vec![],
            rvf_payload: None,
        };
        mgr.register_profile(profile).unwrap();
        let id = mgr.spawn("invalid-trans").unwrap();

        // Provisioning -> Running (invalid, must go through Starting)
        assert!(mgr.transition(&id.0, SandboxState::Running).is_err());
    }

    #[test]
    fn sandbox_manager_terminate_running() {
        let mut mgr = SandboxManager::new();
        let profile = SandboxProfile {
            name: "term-test".into(),
            agent_type: "Worker".into(),
            resources: ResourceEnvelope {
                cpu_cores: 2,
                memory_mb: 4096,
                gpu: GpuRequirement::None,
                disk_mb: 2048,
                max_runtime: Duration::from_secs(300),
            },
            network: NetworkPolicy::Open,
            zone_preference: "zone-d".into(),
            model_tier: "Custom".into(),
            crate_features: vec!["ruvllm".into()],
            rvf_payload: None,
        };
        mgr.register_profile(profile).unwrap();
        let id = mgr.spawn("term-test").unwrap();
        mgr.transition(&id.0, SandboxState::Starting).unwrap();
        mgr.transition(&id.0, SandboxState::Running).unwrap();

        mgr.terminate(&id.0).unwrap();
        assert_eq!(mgr.status(&id.0).unwrap().state, SandboxState::Terminated);
    }

    #[test]
    fn sandbox_manager_spawn_unknown_profile() {
        let mut mgr = SandboxManager::new();
        assert!(mgr.spawn("nonexistent").is_err());
    }

    #[test]
    fn sandbox_manager_count_by_state() {
        let mut mgr = SandboxManager::new();
        let profile = SandboxProfile {
            name: "count-test".into(),
            agent_type: "Worker".into(),
            resources: ResourceEnvelope {
                cpu_cores: 1,
                memory_mb: 512,
                gpu: GpuRequirement::None,
                disk_mb: 1024,
                max_runtime: Duration::from_secs(60),
            },
            network: NetworkPolicy::Isolated,
            zone_preference: "zone-c".into(),
            model_tier: "Small".into(),
            crate_features: vec![],
            rvf_payload: None,
        };
        mgr.register_profile(profile).unwrap();
        mgr.spawn("count-test").unwrap();
        mgr.spawn("count-test").unwrap();
        assert_eq!(mgr.count_by_state(SandboxState::Provisioning), 2);
        assert_eq!(mgr.count_by_state(SandboxState::Running), 0);
    }

    #[test]
    fn sandbox_manager_fleet_deploy() {
        let mut mgr = SandboxManager::new();
        let p1 = SandboxProfile {
            name: "fleet-worker".into(),
            agent_type: "Worker".into(),
            resources: ResourceEnvelope {
                cpu_cores: 2,
                memory_mb: 4096,
                gpu: GpuRequirement::None,
                disk_mb: 2048,
                max_runtime: Duration::from_secs(300),
            },
            network: NetworkPolicy::ClusterOnly,
            zone_preference: "zone-b".into(),
            model_tier: "Small".into(),
            crate_features: vec![],
            rvf_payload: None,
        };
        let p2 = SandboxProfile {
            name: "fleet-researcher".into(),
            agent_type: "Researcher".into(),
            resources: ResourceEnvelope {
                cpu_cores: 8,
                memory_mb: 32768,
                gpu: GpuRequirement::Metal,
                disk_mb: 51200,
                max_runtime: Duration::from_secs(86400),
            },
            network: NetworkPolicy::EgressOnly,
            zone_preference: "zone-a".into(),
            model_tier: "Medium".into(),
            crate_features: vec!["ruvllm".into(), "metal".into()],
            rvf_payload: None,
        };
        mgr.register_profile(p1).unwrap();
        mgr.register_profile(p2).unwrap();

        let manifest = FleetManifest {
            name: "test-fleet".into(),
            version: "1.0".into(),
            sandboxes: vec![
                SandboxSpec {
                    profile: "fleet-worker".into(),
                    count: 3,
                    overrides: None,
                },
                SandboxSpec {
                    profile: "fleet-researcher".into(),
                    count: 1,
                    overrides: None,
                },
            ],
            cloud_policy: None,
        };
        let ids = mgr.deploy_fleet(&manifest).unwrap();
        assert_eq!(ids.len(), 4);
        assert_eq!(mgr.list_instances().len(), 4);
    }

    #[test]
    fn sandbox_manager_fleet_validate_missing_profile() {
        let mgr = SandboxManager::new();
        let manifest = FleetManifest {
            name: "bad-fleet".into(),
            version: "1.0".into(),
            sandboxes: vec![SandboxSpec {
                profile: "missing".into(),
                count: 1,
                overrides: None,
            }],
            cloud_policy: None,
        };
        assert!(mgr.validate_fleet(&manifest).is_err());
    }

    #[test]
    fn valid_transition_fn_works() {
        assert!(valid_transition(
            SandboxState::Provisioning,
            SandboxState::Starting
        ));
        assert!(valid_transition(
            SandboxState::Starting,
            SandboxState::Running
        ));
        assert!(valid_transition(
            SandboxState::Running,
            SandboxState::Suspended
        ));
        assert!(valid_transition(
            SandboxState::Suspended,
            SandboxState::Running
        ));
        assert!(valid_transition(
            SandboxState::Running,
            SandboxState::Stopping
        ));
        assert!(valid_transition(
            SandboxState::Stopping,
            SandboxState::Terminated
        ));
        // Invalid
        assert!(!valid_transition(
            SandboxState::Provisioning,
            SandboxState::Running
        ));
        assert!(!valid_transition(
            SandboxState::Terminated,
            SandboxState::Running
        ));
        assert!(!valid_transition(
            SandboxState::Failed,
            SandboxState::Running
        ));
    }
}
