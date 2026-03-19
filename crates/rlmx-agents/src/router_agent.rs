use tracing::info;

use crate::registry::{PermissionRegistry, SyscallPermission};
use crate::types::{AgentId, AgentStatus, AgentType};

/// Router agent — routes queries to the appropriate strategy or zone.
///
/// Per ADR-005, the Router has minimal permissions: VecSearch for reading,
/// AttentionSelect and HaltCheck for control flow, plus ProcessSend/ProcessRecv
/// for inter-agent messaging. It cannot fork, mutate state, or delete vectors.
pub struct RouterAgent {
    pub id: AgentId,
    pub parent: AgentId,
    pub status: AgentStatus,
    pub queries_routed: u64,
}

impl RouterAgent {
    pub fn new(parent: AgentId) -> Self {
        Self {
            id: AgentId::new(),
            parent,
            status: AgentStatus::Pending,
            queries_routed: 0,
        }
    }

    /// Start the router agent.
    pub fn start(&mut self) {
        self.status = AgentStatus::Running;
        info!(agent_id = %self.id.0, "RouterAgent started");
    }

    /// Route a query to the best strategy, returning the strategy name.
    pub fn route(&mut self, query: &str) -> Result<String, String> {
        if self.status != AgentStatus::Running {
            return Err("RouterAgent is not running".to_string());
        }

        let strategy = if query.len() < 50 {
            "edge"
        } else if query.contains("graph") || query.contains("relation") {
            "graph"
        } else {
            "rlm"
        };

        self.queries_routed += 1;
        info!(
            agent_id = %self.id.0,
            strategy,
            queries_routed = self.queries_routed,
            "Routed query"
        );
        Ok(strategy.to_string())
    }

    /// Terminate the router agent.
    pub fn terminate(&mut self) {
        self.status = AgentStatus::Terminated;
        info!(
            agent_id = %self.id.0,
            queries_routed = self.queries_routed,
            "RouterAgent terminated"
        );
    }

    /// Returns the allowed permissions for this agent type.
    pub fn allowed_permissions() -> Vec<SyscallPermission> {
        PermissionRegistry::permissions_for(AgentType::Router)
            .permissions
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_agent_new() {
        let parent = AgentId::new();
        let agent = RouterAgent::new(parent);
        assert_eq!(agent.status, AgentStatus::Pending);
        assert_eq!(agent.queries_routed, 0);
    }

    #[test]
    fn test_router_agent_start_and_route() {
        let parent = AgentId::new();
        let mut agent = RouterAgent::new(parent);
        agent.start();
        assert_eq!(agent.status, AgentStatus::Running);

        let result = agent.route("short query");
        assert_eq!(result.unwrap(), "edge");
        assert_eq!(agent.queries_routed, 1);
    }

    #[test]
    fn test_router_agent_route_graph() {
        let parent = AgentId::new();
        let mut agent = RouterAgent::new(parent);
        agent.start();

        let long_query = "This is a longer query about graph relationships between entities in the knowledge base";
        let result = agent.route(long_query);
        assert_eq!(result.unwrap(), "graph");
    }

    #[test]
    fn test_router_agent_route_when_not_running() {
        let parent = AgentId::new();
        let mut agent = RouterAgent::new(parent);
        let result = agent.route("test");
        assert!(result.is_err());
    }

    #[test]
    fn test_router_agent_terminate() {
        let parent = AgentId::new();
        let mut agent = RouterAgent::new(parent);
        agent.start();
        agent.terminate();
        assert_eq!(agent.status, AgentStatus::Terminated);
    }

    #[test]
    fn test_router_allowed_permissions() {
        let perms = RouterAgent::allowed_permissions();
        // Router: VecSearch, ProcessSend, ProcessRecv, AttentionSelect, HaltCheck,
        //         VoiceTranscribe, VoiceSynthesize, IntentRoute
        assert_eq!(perms.len(), 8);
    }
}
