//! Role-Based Access Control (RBAC) for RVF operations.
//!
//! Implements the 6-role model from the PRD with default policy mappings.

use serde::{Deserialize, Serialize};

/// The six roles defined by the RLMX security model.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    /// Read-only access to queries and results.
    Viewer,
    /// Can execute queries and view the witness chain.
    Operator,
    /// Can modify parameters and manage plugins.
    Engineer,
    /// Full access except system configuration.
    Admin,
    /// Read-only access to everything including the witness chain.
    Auditor,
    /// Internal kernel operations only.
    System,
}

/// Operations that can be gated by RBAC.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Operation {
    /// Execute a query.
    Query,
    /// Ingest data.
    Ingest,
    /// Manage plugins (install, remove, configure).
    PluginManage,
    /// Modify runtime parameters.
    ParameterModify,
    /// View the witness chain.
    WitnessView,
    /// Export the witness chain.
    WitnessExport,
    /// Change system configuration.
    SystemConfig,
    /// Seal (sign) an RVF container.
    ContainerSeal,
    /// Create a COW branch from a container.
    ContainerBranch,
    /// Merge a branch back into a container.
    ContainerMerge,
}

/// A single policy entry mapping a role to its allowed operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    /// The role this policy applies to.
    pub role: Role,
    /// Operations the role is permitted to perform.
    pub allowed_operations: Vec<Operation>,
}

/// The access control system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControl {
    /// All active policies.
    pub policies: Vec<AccessPolicy>,
}

impl AccessControl {
    /// Create an `AccessControl` instance with the default PRD policies.
    pub fn new() -> Self {
        Self {
            policies: default_policies(),
        }
    }

    /// Check whether the given `role` is allowed to perform `operation`.
    pub fn check(&self, role: &Role, operation: &Operation) -> bool {
        self.policies.iter().any(|p| {
            p.role == *role && p.allowed_operations.contains(operation)
        })
    }
}

impl Default for AccessControl {
    fn default() -> Self {
        Self::new()
    }
}

/// Build the default policy set per PRD specification.
fn default_policies() -> Vec<AccessPolicy> {
    vec![
        AccessPolicy {
            role: Role::Viewer,
            allowed_operations: vec![Operation::Query],
        },
        AccessPolicy {
            role: Role::Operator,
            allowed_operations: vec![
                Operation::Query,
                Operation::Ingest,
                Operation::WitnessView,
            ],
        },
        AccessPolicy {
            role: Role::Engineer,
            allowed_operations: vec![
                Operation::Query,
                Operation::Ingest,
                Operation::PluginManage,
                Operation::ParameterModify,
                Operation::WitnessView,
                Operation::ContainerBranch,
            ],
        },
        AccessPolicy {
            role: Role::Admin,
            allowed_operations: vec![
                Operation::Query,
                Operation::Ingest,
                Operation::PluginManage,
                Operation::ParameterModify,
                Operation::WitnessView,
                Operation::WitnessExport,
                Operation::ContainerSeal,
                Operation::ContainerBranch,
                Operation::ContainerMerge,
            ],
        },
        AccessPolicy {
            role: Role::Auditor,
            allowed_operations: vec![
                Operation::Query,
                Operation::WitnessView,
                Operation::WitnessExport,
            ],
        },
        AccessPolicy {
            role: Role::System,
            allowed_operations: vec![
                Operation::Query,
                Operation::Ingest,
                Operation::PluginManage,
                Operation::ParameterModify,
                Operation::WitnessView,
                Operation::WitnessExport,
                Operation::SystemConfig,
                Operation::ContainerSeal,
                Operation::ContainerBranch,
                Operation::ContainerMerge,
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewer_can_only_query() {
        let ac = AccessControl::new();
        assert!(ac.check(&Role::Viewer, &Operation::Query));
        assert!(!ac.check(&Role::Viewer, &Operation::Ingest));
        assert!(!ac.check(&Role::Viewer, &Operation::PluginManage));
        assert!(!ac.check(&Role::Viewer, &Operation::SystemConfig));
        assert!(!ac.check(&Role::Viewer, &Operation::ContainerSeal));
    }

    #[test]
    fn test_admin_has_broad_access_but_no_system_config() {
        let ac = AccessControl::new();
        assert!(ac.check(&Role::Admin, &Operation::Query));
        assert!(ac.check(&Role::Admin, &Operation::Ingest));
        assert!(ac.check(&Role::Admin, &Operation::ContainerSeal));
        assert!(ac.check(&Role::Admin, &Operation::ContainerMerge));
        // Admin cannot change system config.
        assert!(!ac.check(&Role::Admin, &Operation::SystemConfig));
    }
}
