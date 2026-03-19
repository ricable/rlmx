//! Lifecycle manager for evolved functions.

use std::collections::HashMap;

use chrono::Utc;
use tracing::info;

use crate::types::{
    AutoScoreMode, EvolveError, EvolvedFunction, FunctionId, FunctionStatus,
};

/// Manages the lifecycle of evolved functions.
#[derive(Debug, Default)]
pub struct LifecycleManager {
    functions: HashMap<FunctionId, EvolvedFunction>,
}

impl LifecycleManager {
    /// Create a new empty lifecycle manager.
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// Create a new function in Draft status.
    /// Returns the assigned FunctionId.
    pub fn create(
        &mut self,
        name: &str,
        code: &str,
        goal: &str,
        parent_id: Option<FunctionId>,
    ) -> FunctionId {
        let id = FunctionId::new();
        let version = self
            .by_name(name)
            .iter()
            .map(|f| f.version)
            .max()
            .unwrap_or(0)
            + 1;

        let func = EvolvedFunction {
            id,
            name: name.to_string(),
            version,
            code: code.to_string(),
            goal: goal.to_string(),
            status: FunctionStatus::Draft,
            score_mode: AutoScoreMode::Auto,
            created_at: Utc::now(),
            parent_id,
            metadata: HashMap::new(),
        };

        info!(
            function_id = %id,
            name = name,
            version = version,
            "created evolved function"
        );

        self.functions.insert(id, func);
        id
    }

    /// Transition a function to a new status.
    /// Validates the transition according to the lifecycle state machine.
    pub fn transition(
        &mut self,
        id: FunctionId,
        new_status: FunctionStatus,
    ) -> Result<(), EvolveError> {
        let func = self
            .functions
            .get_mut(&id)
            .ok_or(EvolveError::FunctionNotFound)?;

        if !func.status.can_transition_to(new_status) {
            return Err(EvolveError::InvalidTransition {
                from: func.status,
                to: new_status,
            });
        }

        info!(
            function_id = %id,
            from = %func.status,
            to = %new_status,
            "transitioning function"
        );

        func.status = new_status;
        Ok(())
    }

    /// Get a reference to a function by ID.
    pub fn get(&self, id: FunctionId) -> Option<&EvolvedFunction> {
        self.functions.get(&id)
    }

    /// Get a mutable reference to a function by ID.
    pub fn get_mut(&mut self, id: FunctionId) -> Option<&mut EvolvedFunction> {
        self.functions.get_mut(&id)
    }

    /// List all functions with a given status.
    pub fn list_by_status(&self, status: FunctionStatus) -> Vec<&EvolvedFunction> {
        self.functions
            .values()
            .filter(|f| f.status == status)
            .collect()
    }

    /// List all functions.
    pub fn list_all(&self) -> Vec<&EvolvedFunction> {
        self.functions.values().collect()
    }

    /// Get all versions of a function by name.
    pub fn by_name(&self, name: &str) -> Vec<&EvolvedFunction> {
        self.functions
            .values()
            .filter(|f| f.name == name)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mgr_with_fn() -> (LifecycleManager, FunctionId) {
        let mut mgr = LifecycleManager::new();
        let id = mgr.create("test_fn", "return 1;", "return one", None);
        (mgr, id)
    }

    #[test]
    fn create_sets_draft() {
        let (mgr, id) = mgr_with_fn();
        assert_eq!(mgr.get(id).unwrap().status, FunctionStatus::Draft);
    }

    #[test]
    fn create_sets_version_1() {
        let (mgr, id) = mgr_with_fn();
        assert_eq!(mgr.get(id).unwrap().version, 1);
    }

    #[test]
    fn create_increments_version_by_name() {
        let mut mgr = LifecycleManager::new();
        let id1 = mgr.create("fn_a", "v1", "goal", None);
        let id2 = mgr.create("fn_a", "v2", "goal", Some(id1));
        assert_eq!(mgr.get(id1).unwrap().version, 1);
        assert_eq!(mgr.get(id2).unwrap().version, 2);
    }

    #[test]
    fn create_different_names_independent_versions() {
        let mut mgr = LifecycleManager::new();
        let a = mgr.create("alpha", "code", "goal", None);
        let b = mgr.create("beta", "code", "goal", None);
        assert_eq!(mgr.get(a).unwrap().version, 1);
        assert_eq!(mgr.get(b).unwrap().version, 1);
    }

    #[test]
    fn transition_draft_to_staging() {
        let (mut mgr, id) = mgr_with_fn();
        assert!(mgr.transition(id, FunctionStatus::Staging).is_ok());
        assert_eq!(mgr.get(id).unwrap().status, FunctionStatus::Staging);
    }

    #[test]
    fn transition_staging_to_production() {
        let (mut mgr, id) = mgr_with_fn();
        mgr.transition(id, FunctionStatus::Staging).unwrap();
        assert!(mgr.transition(id, FunctionStatus::Production).is_ok());
        assert_eq!(mgr.get(id).unwrap().status, FunctionStatus::Production);
    }

    #[test]
    fn transition_production_to_deprecated() {
        let (mut mgr, id) = mgr_with_fn();
        mgr.transition(id, FunctionStatus::Staging).unwrap();
        mgr.transition(id, FunctionStatus::Production).unwrap();
        assert!(mgr.transition(id, FunctionStatus::Deprecated).is_ok());
    }

    #[test]
    fn transition_any_to_killed() {
        for initial_target in [FunctionStatus::Staging] {
            let (mut mgr, id) = mgr_with_fn();
            mgr.transition(id, initial_target).unwrap();
            assert!(mgr.transition(id, FunctionStatus::Killed).is_ok());
        }
    }

    #[test]
    fn transition_draft_to_killed() {
        let (mut mgr, id) = mgr_with_fn();
        assert!(mgr.transition(id, FunctionStatus::Killed).is_ok());
    }

    #[test]
    fn transition_invalid_skip() {
        let (mut mgr, id) = mgr_with_fn();
        let result = mgr.transition(id, FunctionStatus::Production);
        assert!(result.is_err());
        match result.unwrap_err() {
            EvolveError::InvalidTransition { from, to } => {
                assert_eq!(from, FunctionStatus::Draft);
                assert_eq!(to, FunctionStatus::Production);
            }
            _ => panic!("expected InvalidTransition"),
        }
    }

    #[test]
    fn transition_backward_staging_to_draft() {
        let (mut mgr, id) = mgr_with_fn();
        mgr.transition(id, FunctionStatus::Staging).unwrap();
        assert!(mgr.transition(id, FunctionStatus::Draft).is_err());
    }

    #[test]
    fn transition_backward_production_to_staging() {
        let (mut mgr, id) = mgr_with_fn();
        mgr.transition(id, FunctionStatus::Staging).unwrap();
        mgr.transition(id, FunctionStatus::Production).unwrap();
        assert!(mgr.transition(id, FunctionStatus::Staging).is_err());
    }

    #[test]
    fn transition_killed_cannot_revive() {
        let (mut mgr, id) = mgr_with_fn();
        mgr.transition(id, FunctionStatus::Killed).unwrap();
        assert!(mgr.transition(id, FunctionStatus::Draft).is_err());
        assert!(mgr.transition(id, FunctionStatus::Staging).is_err());
    }

    #[test]
    fn transition_not_found() {
        let mut mgr = LifecycleManager::new();
        let fake = FunctionId::new();
        let result = mgr.transition(fake, FunctionStatus::Staging);
        assert!(matches!(result, Err(EvolveError::FunctionNotFound)));
    }

    #[test]
    fn get_nonexistent() {
        let mgr = LifecycleManager::new();
        assert!(mgr.get(FunctionId::new()).is_none());
    }

    #[test]
    fn get_mut_works() {
        let (mut mgr, id) = mgr_with_fn();
        mgr.get_mut(id).unwrap().code = "return 2;".into();
        assert_eq!(mgr.get(id).unwrap().code, "return 2;");
    }

    #[test]
    fn list_by_status_filters() {
        let mut mgr = LifecycleManager::new();
        let a = mgr.create("a", "code", "goal", None);
        let _b = mgr.create("b", "code", "goal", None);
        mgr.transition(a, FunctionStatus::Staging).unwrap();

        let drafts = mgr.list_by_status(FunctionStatus::Draft);
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].name, "b");

        let staging = mgr.list_by_status(FunctionStatus::Staging);
        assert_eq!(staging.len(), 1);
        assert_eq!(staging[0].name, "a");
    }

    #[test]
    fn list_all() {
        let mut mgr = LifecycleManager::new();
        mgr.create("a", "code", "goal", None);
        mgr.create("b", "code", "goal", None);
        assert_eq!(mgr.list_all().len(), 2);
    }

    #[test]
    fn by_name_returns_all_versions() {
        let mut mgr = LifecycleManager::new();
        let id1 = mgr.create("shared", "v1", "goal", None);
        mgr.create("shared", "v2", "goal", Some(id1));
        mgr.create("other", "v1", "goal", None);
        assert_eq!(mgr.by_name("shared").len(), 2);
        assert_eq!(mgr.by_name("other").len(), 1);
        assert_eq!(mgr.by_name("nonexistent").len(), 0);
    }

    #[test]
    fn create_stores_parent_id() {
        let mut mgr = LifecycleManager::new();
        let parent = mgr.create("fn", "v1", "goal", None);
        let child = mgr.create("fn", "v2", "goal", Some(parent));
        assert_eq!(mgr.get(child).unwrap().parent_id, Some(parent));
        assert_eq!(mgr.get(parent).unwrap().parent_id, None);
    }

    #[test]
    fn create_stores_code_and_goal() {
        let (mgr, id) = mgr_with_fn();
        let f = mgr.get(id).unwrap();
        assert_eq!(f.code, "return 1;");
        assert_eq!(f.goal, "return one");
    }

    #[test]
    fn deprecated_to_killed() {
        let (mut mgr, id) = mgr_with_fn();
        mgr.transition(id, FunctionStatus::Staging).unwrap();
        mgr.transition(id, FunctionStatus::Production).unwrap();
        mgr.transition(id, FunctionStatus::Deprecated).unwrap();
        assert!(mgr.transition(id, FunctionStatus::Killed).is_ok());
    }
}
