use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::types::AgentType;

// Re-export SyscallPermission from the kernel instead of duplicating it.
pub use rlmx_kernel::types::SyscallPermission;

/// Returns all 12 concrete syscall permission variants (excludes `All`).
pub fn all_concrete_permissions() -> &'static [SyscallPermission] {
    &[
        SyscallPermission::VecInsert,
        SyscallPermission::VecSearch,
        SyscallPermission::VecDelete,
        SyscallPermission::GraphQuery,
        SyscallPermission::GraphCut,
        SyscallPermission::GraphDiffuse,
        SyscallPermission::ProcessFork,
        SyscallPermission::ProcessSend,
        SyscallPermission::ProcessRecv,
        SyscallPermission::StateMutate,
        SyscallPermission::AttentionSelect,
        SyscallPermission::HaltCheck,
    ]
}

/// A set of syscall permissions granted to an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPermissions {
    pub permissions: HashSet<SyscallPermission>,
}

impl AgentPermissions {
    pub fn new(perms: impl IntoIterator<Item = SyscallPermission>) -> Self {
        Self {
            permissions: perms.into_iter().collect(),
        }
    }

    pub fn has(&self, perm: SyscallPermission) -> bool {
        self.permissions.contains(&perm)
    }

    pub fn grant(&mut self, perm: SyscallPermission) {
        self.permissions.insert(perm);
    }

    pub fn revoke(&mut self, perm: SyscallPermission) {
        self.permissions.remove(&perm);
    }

    pub fn count(&self) -> usize {
        self.permissions.len()
    }
}

/// The 12x12 permission matrix type (agent_type_index × syscall_permission_index).
pub type PermissionMatrix = [[bool; 12]; 12];

/// Static permission registry mapping each AgentType to allowed syscalls.
pub struct PermissionRegistry;

impl PermissionRegistry {
    /// Returns the permissions for a given agent type (the 12x12 matrix).
    pub fn permissions_for(agent_type: AgentType) -> AgentPermissions {
        use SyscallPermission::*;
        let perms: Vec<SyscallPermission> = match agent_type {
            // Coordinator: full access (PID 0)
            AgentType::Coordinator => all_concrete_permissions().to_vec(),
            // Researcher: read vectors, graph queries, fork, messaging, attention
            AgentType::Researcher => vec![
                VecInsert,
                VecSearch,
                GraphQuery,
                GraphDiffuse,
                ProcessFork,
                ProcessSend,
                ProcessRecv,
                AttentionSelect,
            ],
            // Router: read-only routing, messaging, attention, halt (no fork, no graph per ADR-005)
            AgentType::Router => vec![
                VecSearch,
                ProcessSend,
                ProcessRecv,
                AttentionSelect,
                HaltCheck,
            ],
            // Experimenter: broad access for experiments, can fork workers
            AgentType::Experimenter => vec![
                VecInsert,
                VecSearch,
                VecDelete,
                GraphQuery,
                GraphDiffuse,
                ProcessFork,
                ProcessSend,
                ProcessRecv,
                StateMutate,
                AttentionSelect,
            ],
            // Worker: execute tasks, messaging, state mutation
            AgentType::Worker => vec![
                VecInsert,
                VecSearch,
                VecDelete,
                ProcessSend,
                ProcessRecv,
                StateMutate,
            ],
            // Monitor: read-only observation, messaging, halt check
            AgentType::Monitor => vec![
                VecSearch,
                GraphQuery,
                ProcessSend,
                ProcessRecv,
                AttentionSelect,
                HaltCheck,
            ],
            // Reviewer: read access, messaging, attention
            AgentType::Reviewer => vec![
                VecSearch,
                GraphQuery,
                GraphDiffuse,
                ProcessSend,
                ProcessRecv,
                AttentionSelect,
                StateMutate,
            ],
            // Trainer: vector ops, state mutation, messaging
            AgentType::Trainer => vec![
                VecInsert,
                VecSearch,
                VecDelete,
                ProcessSend,
                ProcessRecv,
                StateMutate,
                AttentionSelect,
            ],
            // Validator: read-only verification, no state mutation
            AgentType::Validator => {
                vec![VecSearch, GraphQuery, ProcessSend, ProcessRecv, HaltCheck]
            }
            // Replicator: vector ops for sync, messaging
            AgentType::Replicator => vec![
                VecInsert,
                VecSearch,
                VecDelete,
                ProcessSend,
                ProcessRecv,
                StateMutate,
            ],
            // Embedder: vector insert/search only, no state mutation
            AgentType::Embedder => vec![VecInsert, VecSearch, ProcessSend, ProcessRecv],
            // Analyst: graph operations, vector search, attention
            AgentType::Analyst => vec![
                VecSearch,
                GraphQuery,
                GraphCut,
                GraphDiffuse,
                ProcessSend,
                ProcessRecv,
                AttentionSelect,
                StateMutate,
            ],
        };
        AgentPermissions::new(perms)
    }

    /// Whether `parent` can spawn `child` based on the hierarchy.
    pub fn can_spawn(parent: AgentType, child: AgentType) -> bool {
        let allowed = Self::spawn_hierarchy();
        allowed
            .get(&parent)
            .map(|children| children.contains(&child))
            .unwrap_or(false)
    }

    /// Returns the valid parent -> children spawn map.
    pub fn spawn_hierarchy() -> HashMap<AgentType, Vec<AgentType>> {
        let mut map = HashMap::new();

        // Coordinator can spawn anything except another Coordinator
        map.insert(
            AgentType::Coordinator,
            vec![
                AgentType::Researcher,
                AgentType::Router,
                AgentType::Experimenter,
                AgentType::Worker,
                AgentType::Monitor,
                AgentType::Reviewer,
                AgentType::Trainer,
                AgentType::Validator,
                AgentType::Replicator,
                AgentType::Embedder,
                AgentType::Analyst,
            ],
        );

        // Researcher can spawn Workers and Experimenters
        map.insert(
            AgentType::Researcher,
            vec![
                AgentType::Worker,
                AgentType::Experimenter,
                AgentType::Embedder,
            ],
        );

        // Router can spawn Workers
        map.insert(AgentType::Router, vec![AgentType::Worker]);

        // Experimenter can spawn Workers
        map.insert(AgentType::Experimenter, vec![AgentType::Worker]);

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_has_all_permissions() {
        let perms = PermissionRegistry::permissions_for(AgentType::Coordinator);
        assert_eq!(perms.count(), 12);
        for p in all_concrete_permissions() {
            assert!(perms.has(*p), "Coordinator missing {:?}", p);
        }
    }

    #[test]
    fn test_router_cannot_mutate_state() {
        let perms = PermissionRegistry::permissions_for(AgentType::Router);
        assert!(!perms.has(SyscallPermission::StateMutate));
    }

    #[test]
    fn test_validator_cannot_mutate_state() {
        let perms = PermissionRegistry::permissions_for(AgentType::Validator);
        assert!(!perms.has(SyscallPermission::StateMutate));
    }

    #[test]
    fn test_embedder_limited_permissions() {
        let perms = PermissionRegistry::permissions_for(AgentType::Embedder);
        assert!(perms.has(SyscallPermission::VecInsert));
        assert!(perms.has(SyscallPermission::VecSearch));
        assert!(!perms.has(SyscallPermission::StateMutate));
        assert!(!perms.has(SyscallPermission::GraphQuery));
        assert!(!perms.has(SyscallPermission::ProcessFork));
    }

    #[test]
    fn test_only_fork_agents_have_process_fork() {
        for agent_type in AgentType::all() {
            let perms = PermissionRegistry::permissions_for(*agent_type);
            if agent_type.can_fork() {
                assert!(
                    perms.has(SyscallPermission::ProcessFork),
                    "{agent_type} can_fork but lacks ProcessFork permission"
                );
            }
        }
    }

    #[test]
    fn test_spawn_hierarchy_coordinator() {
        // Coordinator can spawn all except Coordinator
        for agent_type in AgentType::all() {
            if *agent_type == AgentType::Coordinator {
                assert!(!PermissionRegistry::can_spawn(
                    AgentType::Coordinator,
                    *agent_type
                ));
            } else {
                assert!(
                    PermissionRegistry::can_spawn(AgentType::Coordinator, *agent_type),
                    "Coordinator should be able to spawn {agent_type}"
                );
            }
        }
    }

    #[test]
    fn test_spawn_hierarchy_researcher() {
        assert!(PermissionRegistry::can_spawn(
            AgentType::Researcher,
            AgentType::Worker
        ));
        assert!(PermissionRegistry::can_spawn(
            AgentType::Researcher,
            AgentType::Experimenter
        ));
        assert!(PermissionRegistry::can_spawn(
            AgentType::Researcher,
            AgentType::Embedder
        ));
        assert!(!PermissionRegistry::can_spawn(
            AgentType::Researcher,
            AgentType::Coordinator
        ));
        assert!(!PermissionRegistry::can_spawn(
            AgentType::Researcher,
            AgentType::Analyst
        ));
    }

    #[test]
    fn test_spawn_hierarchy_non_forking() {
        // Non-forking agents cannot spawn anything
        assert!(!PermissionRegistry::can_spawn(
            AgentType::Worker,
            AgentType::Worker
        ));
        assert!(!PermissionRegistry::can_spawn(
            AgentType::Monitor,
            AgentType::Worker
        ));
        assert!(!PermissionRegistry::can_spawn(
            AgentType::Validator,
            AgentType::Worker
        ));
    }

    #[test]
    fn test_agent_permissions_grant_revoke() {
        let mut perms = AgentPermissions::new(vec![SyscallPermission::VecSearch]);
        assert!(perms.has(SyscallPermission::VecSearch));
        assert!(!perms.has(SyscallPermission::VecInsert));

        perms.grant(SyscallPermission::VecInsert);
        assert!(perms.has(SyscallPermission::VecInsert));
        assert_eq!(perms.count(), 2);

        perms.revoke(SyscallPermission::VecSearch);
        assert!(!perms.has(SyscallPermission::VecSearch));
        assert_eq!(perms.count(), 1);
    }

    #[test]
    fn test_all_agent_types_have_messaging() {
        for agent_type in AgentType::all() {
            let perms = PermissionRegistry::permissions_for(*agent_type);
            assert!(
                perms.has(SyscallPermission::ProcessSend),
                "{agent_type} missing ProcessSend"
            );
            assert!(
                perms.has(SyscallPermission::ProcessRecv),
                "{agent_type} missing ProcessRecv"
            );
        }
    }

    #[test]
    fn test_all_agent_types_have_vec_search() {
        for agent_type in AgentType::all() {
            let perms = PermissionRegistry::permissions_for(*agent_type);
            assert!(
                perms.has(SyscallPermission::VecSearch),
                "{agent_type} missing VecSearch"
            );
        }
    }
}
