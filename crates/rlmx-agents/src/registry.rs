use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::types::AgentType;

// Re-export SyscallPermission from the kernel instead of duplicating it.
pub use rlmx_kernel::types::SyscallPermission;

/// Returns all 18 concrete syscall permission variants (excludes `All`).
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
        SyscallPermission::VoiceTranscribe,
        SyscallPermission::VoiceSynthesize,
        SyscallPermission::IntentRoute,
        SyscallPermission::MeshSync,
        SyscallPermission::FederationContribute,
        SyscallPermission::ArtifactWrite,
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

/// Static permission registry mapping each AgentType to allowed syscalls.
pub struct PermissionRegistry;

impl PermissionRegistry {
    /// Returns the permissions for a given agent type (the 17x17 matrix).
    pub fn permissions_for(agent_type: AgentType) -> AgentPermissions {
        use SyscallPermission::*;
        let perms: Vec<SyscallPermission> = match agent_type {
            // Coordinator: full access (PID 0)
            AgentType::Coordinator => all_concrete_permissions().to_vec(),
            // Researcher: read vectors, graph queries, fork, messaging, attention, voice synth, intent, artifact write
            AgentType::Researcher => vec![
                VecInsert,
                VecSearch,
                GraphQuery,
                GraphDiffuse,
                ProcessFork,
                ProcessSend,
                ProcessRecv,
                AttentionSelect,
                VoiceSynthesize,
                IntentRoute,
                ArtifactWrite,
            ],
            // Router: read-only routing, messaging, attention, halt, voice transcribe/synth, intent
            AgentType::Router => vec![
                VecSearch,
                ProcessSend,
                ProcessRecv,
                AttentionSelect,
                HaltCheck,
                VoiceTranscribe,
                VoiceSynthesize,
                IntentRoute,
            ],
            // Experimenter: broad access for experiments, can fork workers, artifact write
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
                VoiceSynthesize,
                IntentRoute,
                ArtifactWrite,
            ],
            // Worker: execute tasks, messaging, state mutation, voice synth, intent
            AgentType::Worker => vec![
                VecInsert,
                VecSearch,
                VecDelete,
                ProcessSend,
                ProcessRecv,
                StateMutate,
                VoiceSynthesize,
                IntentRoute,
            ],
            // Monitor: read-only observation, messaging, halt check, voice synth, intent
            AgentType::Monitor => vec![
                VecSearch,
                GraphQuery,
                ProcessSend,
                ProcessRecv,
                AttentionSelect,
                HaltCheck,
                VoiceSynthesize,
                IntentRoute,
            ],
            // Reviewer: read access, messaging, attention, voice synth, intent
            AgentType::Reviewer => vec![
                VecSearch,
                GraphQuery,
                GraphDiffuse,
                ProcessSend,
                ProcessRecv,
                AttentionSelect,
                StateMutate,
                VoiceSynthesize,
                IntentRoute,
            ],
            // Trainer: vector ops, state mutation, messaging, voice synth, intent
            AgentType::Trainer => vec![
                VecInsert,
                VecSearch,
                VecDelete,
                ProcessSend,
                ProcessRecv,
                StateMutate,
                AttentionSelect,
                VoiceSynthesize,
                IntentRoute,
            ],
            // Validator: read-only verification, no state mutation, voice synth, intent
            AgentType::Validator => vec![
                VecSearch,
                GraphQuery,
                ProcessSend,
                ProcessRecv,
                HaltCheck,
                VoiceSynthesize,
                IntentRoute,
            ],
            // Replicator: vector ops for sync, messaging, voice synth, intent
            AgentType::Replicator => vec![
                VecInsert,
                VecSearch,
                VecDelete,
                ProcessSend,
                ProcessRecv,
                StateMutate,
                VoiceSynthesize,
                IntentRoute,
            ],
            // Embedder: vector insert/search only, no state mutation, voice synth, intent
            AgentType::Embedder => vec![
                VecInsert,
                VecSearch,
                ProcessSend,
                ProcessRecv,
                VoiceSynthesize,
                IntentRoute,
            ],
            // Analyst: graph operations, vector search, attention, voice synth, intent
            AgentType::Analyst => vec![
                VecSearch,
                GraphQuery,
                GraphCut,
                GraphDiffuse,
                ProcessSend,
                ProcessRecv,
                AttentionSelect,
                StateMutate,
                VoiceSynthesize,
                IntentRoute,
            ],
            // VoiceCoordinator: full voice access, fork, messaging, attention, intent
            AgentType::VoiceCoordinator => vec![
                VecInsert,
                VecSearch,
                ProcessFork,
                ProcessSend,
                ProcessRecv,
                AttentionSelect,
                VoiceTranscribe,
                VoiceSynthesize,
                IntentRoute,
            ],
            // MarketplaceManager: state mutation, vector ops, messaging, voice synth, intent
            AgentType::MarketplaceManager => vec![
                VecInsert,
                VecSearch,
                VecDelete,
                ProcessSend,
                ProcessRecv,
                StateMutate,
                VoiceSynthesize,
                IntentRoute,
            ],
            // MeshCoordinator: graph, vectors, fork, messaging, attention, mesh sync, voice synth, intent
            AgentType::MeshCoordinator => vec![
                VecInsert,
                VecSearch,
                GraphQuery,
                GraphDiffuse,
                ProcessFork,
                ProcessSend,
                ProcessRecv,
                AttentionSelect,
                MeshSync,
                VoiceSynthesize,
                IntentRoute,
            ],
            // FederationAgent: search, messaging, attention, federation, voice synth, intent
            AgentType::FederationAgent => vec![
                VecSearch,
                ProcessSend,
                ProcessRecv,
                AttentionSelect,
                FederationContribute,
                VoiceSynthesize,
                IntentRoute,
            ],
            // BillingManager: vector ops, messaging, state mutation, voice synth, intent
            AgentType::BillingManager => vec![
                VecInsert,
                VecSearch,
                VecDelete,
                ProcessSend,
                ProcessRecv,
                StateMutate,
                VoiceSynthesize,
                IntentRoute,
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
                AgentType::VoiceCoordinator,
                AgentType::MarketplaceManager,
                AgentType::MeshCoordinator,
                AgentType::FederationAgent,
                AgentType::BillingManager,
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

        // Experimenter can spawn Workers
        map.insert(AgentType::Experimenter, vec![AgentType::Worker]);

        // VoiceCoordinator can spawn Workers and Embedders for voice processing
        map.insert(
            AgentType::VoiceCoordinator,
            vec![AgentType::Worker, AgentType::Embedder],
        );

        // MeshCoordinator can spawn Workers for mesh tasks
        map.insert(AgentType::MeshCoordinator, vec![AgentType::Worker]);

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinator_has_all_permissions() {
        let perms = PermissionRegistry::permissions_for(AgentType::Coordinator);
        assert_eq!(perms.count(), 18);
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
            let has_fork_perm = perms.has(SyscallPermission::ProcessFork);
            let can_fork = agent_type.can_fork();
            assert_eq!(
                has_fork_perm, can_fork,
                "{agent_type}: ProcessFork permission ({has_fork_perm}) != can_fork ({can_fork})"
            );
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

    #[test]
    fn test_all_agent_types_have_voice_synthesize_and_intent_route() {
        for agent_type in AgentType::all() {
            let perms = PermissionRegistry::permissions_for(*agent_type);
            assert!(
                perms.has(SyscallPermission::VoiceSynthesize),
                "{agent_type} missing VoiceSynthesize"
            );
            assert!(
                perms.has(SyscallPermission::IntentRoute),
                "{agent_type} missing IntentRoute"
            );
        }
    }

    #[test]
    fn test_voice_coordinator_permissions() {
        let perms = PermissionRegistry::permissions_for(AgentType::VoiceCoordinator);
        assert!(perms.has(SyscallPermission::VoiceTranscribe));
        assert!(perms.has(SyscallPermission::VoiceSynthesize));
        assert!(perms.has(SyscallPermission::IntentRoute));
        assert!(perms.has(SyscallPermission::ProcessFork));
        assert!(perms.has(SyscallPermission::AttentionSelect));
        assert!(!perms.has(SyscallPermission::StateMutate));
    }

    #[test]
    fn test_marketplace_manager_permissions() {
        let perms = PermissionRegistry::permissions_for(AgentType::MarketplaceManager);
        assert!(perms.has(SyscallPermission::StateMutate));
        assert!(perms.has(SyscallPermission::VecInsert));
        assert!(perms.has(SyscallPermission::VecDelete));
        assert!(perms.has(SyscallPermission::VoiceSynthesize));
        assert!(perms.has(SyscallPermission::IntentRoute));
        assert!(!perms.has(SyscallPermission::ProcessFork));
        assert!(!perms.has(SyscallPermission::VoiceTranscribe));
    }

    #[test]
    fn test_voice_transcribe_restricted() {
        // Only Coordinator, Router, and VoiceCoordinator should have VoiceTranscribe
        for agent_type in AgentType::all() {
            let perms = PermissionRegistry::permissions_for(*agent_type);
            let should_have = matches!(
                agent_type,
                AgentType::Coordinator | AgentType::Router | AgentType::VoiceCoordinator
            );
            assert_eq!(
                perms.has(SyscallPermission::VoiceTranscribe),
                should_have,
                "{agent_type} VoiceTranscribe mismatch"
            );
        }
    }

    #[test]
    fn test_mesh_coordinator_permissions() {
        let perms = PermissionRegistry::permissions_for(AgentType::MeshCoordinator);
        assert!(perms.has(SyscallPermission::MeshSync));
        assert!(perms.has(SyscallPermission::ProcessFork));
        assert!(perms.has(SyscallPermission::GraphQuery));
        assert!(perms.has(SyscallPermission::VoiceSynthesize));
        assert!(perms.has(SyscallPermission::IntentRoute));
        assert!(!perms.has(SyscallPermission::VoiceTranscribe));
        assert!(!perms.has(SyscallPermission::StateMutate));
    }

    #[test]
    fn test_federation_agent_permissions() {
        let perms = PermissionRegistry::permissions_for(AgentType::FederationAgent);
        assert!(perms.has(SyscallPermission::FederationContribute));
        assert!(perms.has(SyscallPermission::VecSearch));
        assert!(perms.has(SyscallPermission::AttentionSelect));
        assert!(perms.has(SyscallPermission::VoiceSynthesize));
        assert!(perms.has(SyscallPermission::IntentRoute));
        assert!(!perms.has(SyscallPermission::VoiceTranscribe));
        assert!(!perms.has(SyscallPermission::StateMutate));
        assert!(!perms.has(SyscallPermission::ProcessFork));
    }

    #[test]
    fn test_billing_manager_permissions() {
        let perms = PermissionRegistry::permissions_for(AgentType::BillingManager);
        assert!(perms.has(SyscallPermission::StateMutate));
        assert!(perms.has(SyscallPermission::VecInsert));
        assert!(perms.has(SyscallPermission::VecDelete));
        assert!(perms.has(SyscallPermission::VoiceSynthesize));
        assert!(perms.has(SyscallPermission::IntentRoute));
        assert!(!perms.has(SyscallPermission::VoiceTranscribe));
        assert!(!perms.has(SyscallPermission::ProcessFork));
        assert!(!perms.has(SyscallPermission::MeshSync));
    }

    #[test]
    fn test_new_types_no_voice_transcribe() {
        // New agent types should NOT have VoiceTranscribe
        assert!(
            !PermissionRegistry::permissions_for(AgentType::MeshCoordinator)
                .has(SyscallPermission::VoiceTranscribe)
        );
        assert!(
            !PermissionRegistry::permissions_for(AgentType::FederationAgent)
                .has(SyscallPermission::VoiceTranscribe)
        );
        assert!(
            !PermissionRegistry::permissions_for(AgentType::BillingManager)
                .has(SyscallPermission::VoiceTranscribe)
        );
    }

    #[test]
    fn test_spawn_hierarchy_mesh_coordinator() {
        assert!(PermissionRegistry::can_spawn(
            AgentType::MeshCoordinator,
            AgentType::Worker
        ));
        assert!(!PermissionRegistry::can_spawn(
            AgentType::MeshCoordinator,
            AgentType::Coordinator
        ));
    }

    #[test]
    fn test_spawn_hierarchy_voice_coordinator() {
        assert!(PermissionRegistry::can_spawn(
            AgentType::VoiceCoordinator,
            AgentType::Worker
        ));
        assert!(PermissionRegistry::can_spawn(
            AgentType::VoiceCoordinator,
            AgentType::Embedder
        ));
        assert!(!PermissionRegistry::can_spawn(
            AgentType::VoiceCoordinator,
            AgentType::Coordinator
        ));
    }
}
