use crate::types::AgentStatus;

/// Validates agent state transitions per ADR-005 state machine:
///   Pending -> Starting -> Running -> Paused -> Running (resume)
///                                  -> Stopping -> Terminated
///                       -> Failed (from Starting or Running)
///
/// Invalid transitions are rejected, preventing agents from skipping states
/// or resurrecting after termination.
pub struct AgentLifecycle;

impl AgentLifecycle {
    /// Returns true if transitioning from `current` to `next` is valid.
    pub fn is_valid_transition(current: AgentStatus, next: AgentStatus) -> bool {
        matches!(
            (current, next),
            // Forward flow
            (AgentStatus::Pending, AgentStatus::Starting)
                | (AgentStatus::Starting, AgentStatus::Running)
                | (AgentStatus::Running, AgentStatus::Paused)
                | (AgentStatus::Running, AgentStatus::Stopping)
                | (AgentStatus::Paused, AgentStatus::Running)
                | (AgentStatus::Paused, AgentStatus::Stopping)
                | (AgentStatus::Stopping, AgentStatus::Terminated)
                // Failure paths
                | (AgentStatus::Starting, AgentStatus::Failed)
                | (AgentStatus::Running, AgentStatus::Failed)
        )
    }

    /// Attempts a transition, returning Ok(next) if valid or Err with reason.
    pub fn transition(current: AgentStatus, next: AgentStatus) -> Result<AgentStatus, String> {
        if Self::is_valid_transition(current, next) {
            Ok(next)
        } else {
            Err(format!("Invalid state transition: {current:?} -> {next:?}"))
        }
    }

    /// Returns all valid next states from the given state.
    pub fn valid_next_states(current: AgentStatus) -> Vec<AgentStatus> {
        let all = [
            AgentStatus::Pending,
            AgentStatus::Starting,
            AgentStatus::Running,
            AgentStatus::Paused,
            AgentStatus::Stopping,
            AgentStatus::Terminated,
            AgentStatus::Failed,
        ];
        all.iter()
            .copied()
            .filter(|next| Self::is_valid_transition(current, *next))
            .collect()
    }

    /// Returns true if the agent is in a terminal state (no further transitions).
    pub fn is_terminal(status: AgentStatus) -> bool {
        matches!(status, AgentStatus::Terminated | AgentStatus::Failed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_forward_flow() {
        assert!(AgentLifecycle::is_valid_transition(
            AgentStatus::Pending,
            AgentStatus::Starting
        ));
        assert!(AgentLifecycle::is_valid_transition(
            AgentStatus::Starting,
            AgentStatus::Running
        ));
        assert!(AgentLifecycle::is_valid_transition(
            AgentStatus::Running,
            AgentStatus::Stopping
        ));
        assert!(AgentLifecycle::is_valid_transition(
            AgentStatus::Stopping,
            AgentStatus::Terminated
        ));
    }

    #[test]
    fn test_pause_resume() {
        assert!(AgentLifecycle::is_valid_transition(
            AgentStatus::Running,
            AgentStatus::Paused
        ));
        assert!(AgentLifecycle::is_valid_transition(
            AgentStatus::Paused,
            AgentStatus::Running
        ));
        assert!(AgentLifecycle::is_valid_transition(
            AgentStatus::Paused,
            AgentStatus::Stopping
        ));
    }

    #[test]
    fn test_failure_paths() {
        assert!(AgentLifecycle::is_valid_transition(
            AgentStatus::Starting,
            AgentStatus::Failed
        ));
        assert!(AgentLifecycle::is_valid_transition(
            AgentStatus::Running,
            AgentStatus::Failed
        ));
        // Cannot fail from Pending
        assert!(!AgentLifecycle::is_valid_transition(
            AgentStatus::Pending,
            AgentStatus::Failed
        ));
    }

    #[test]
    fn test_invalid_transitions() {
        // Cannot skip states
        assert!(!AgentLifecycle::is_valid_transition(
            AgentStatus::Pending,
            AgentStatus::Running
        ));
        // Cannot go backwards
        assert!(!AgentLifecycle::is_valid_transition(
            AgentStatus::Running,
            AgentStatus::Starting
        ));
        // Cannot resurrect from terminal
        assert!(!AgentLifecycle::is_valid_transition(
            AgentStatus::Terminated,
            AgentStatus::Running
        ));
        assert!(!AgentLifecycle::is_valid_transition(
            AgentStatus::Failed,
            AgentStatus::Running
        ));
    }

    #[test]
    fn test_transition_ok() {
        let result = AgentLifecycle::transition(AgentStatus::Pending, AgentStatus::Starting);
        assert_eq!(result.unwrap(), AgentStatus::Starting);
    }

    #[test]
    fn test_transition_err() {
        let result = AgentLifecycle::transition(AgentStatus::Terminated, AgentStatus::Running);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_next_states() {
        let next = AgentLifecycle::valid_next_states(AgentStatus::Running);
        assert!(next.contains(&AgentStatus::Paused));
        assert!(next.contains(&AgentStatus::Stopping));
        assert!(next.contains(&AgentStatus::Failed));
        assert_eq!(next.len(), 3);
    }

    #[test]
    fn test_terminal_states() {
        assert!(AgentLifecycle::is_terminal(AgentStatus::Terminated));
        assert!(AgentLifecycle::is_terminal(AgentStatus::Failed));
        assert!(!AgentLifecycle::is_terminal(AgentStatus::Running));
        assert!(!AgentLifecycle::is_terminal(AgentStatus::Pending));
    }

    #[test]
    fn test_terminal_states_have_no_next() {
        assert!(AgentLifecycle::valid_next_states(AgentStatus::Terminated).is_empty());
        assert!(AgentLifecycle::valid_next_states(AgentStatus::Failed).is_empty());
    }
}
