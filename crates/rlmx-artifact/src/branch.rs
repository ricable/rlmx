use std::collections::HashMap;
use tracing::debug;

use crate::dag::ArtifactDag;
use crate::types::{ArtifactError, ArtifactId};

/// Manages named branches pointing to artifact heads in the DAG.
#[derive(Debug, Default)]
pub struct BranchManager {
    branches: HashMap<String, ArtifactId>,
}

impl BranchManager {
    /// Create a new empty branch manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new branch pointing to the given head artifact.
    /// The head must exist in the provided DAG.
    pub fn create_branch(
        &mut self,
        name: &str,
        head_id: ArtifactId,
        dag: &ArtifactDag,
    ) -> Result<(), ArtifactError> {
        if self.branches.contains_key(name) {
            return Err(ArtifactError::BranchConflict(format!(
                "branch '{}' already exists",
                name
            )));
        }
        if !dag.contains(&head_id) {
            return Err(ArtifactError::NotFound(format!(
                "head artifact {} not found in DAG",
                head_id
            )));
        }

        debug!(branch = name, head = %head_id, "created branch");
        self.branches.insert(name.to_string(), head_id);
        Ok(())
    }

    /// Advance a branch to a new head. Returns (old_head, new_head).
    /// The new head must exist in the provided DAG.
    pub fn advance(
        &mut self,
        name: &str,
        new_head_id: ArtifactId,
        dag: &ArtifactDag,
    ) -> Result<(ArtifactId, ArtifactId), ArtifactError> {
        let old_head = *self
            .branches
            .get(name)
            .ok_or_else(|| ArtifactError::NotFound(format!("branch '{}' not found", name)))?;

        if !dag.contains(&new_head_id) {
            return Err(ArtifactError::NotFound(format!(
                "new head artifact {} not found in DAG",
                new_head_id
            )));
        }

        debug!(branch = name, old_head = %old_head, new_head = %new_head_id, "advanced branch");
        self.branches.insert(name.to_string(), new_head_id);
        Ok((old_head, new_head_id))
    }

    /// Get the head artifact ID for a branch.
    pub fn get_head(&self, name: &str) -> Option<ArtifactId> {
        self.branches.get(name).copied()
    }

    /// List all branch names.
    pub fn list_branches(&self) -> Vec<&str> {
        self.branches.keys().map(|s| s.as_str()).collect()
    }

    /// Delete a branch. Returns the head ID that was removed.
    pub fn delete_branch(&mut self, name: &str) -> Result<ArtifactId, ArtifactError> {
        self.branches
            .remove(name)
            .ok_or_else(|| ArtifactError::NotFound(format!("branch '{}' not found", name)))
    }

    /// Number of branches.
    pub fn len(&self) -> usize {
        self.branches.len()
    }

    /// Whether there are no branches.
    pub fn is_empty(&self) -> bool {
        self.branches.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Artifact;
    use chrono::Utc;

    fn make_dag_with_artifacts(id_bytes: &[u8]) -> (ArtifactDag, Vec<ArtifactId>) {
        let mut dag = ArtifactDag::new();
        let mut ids = Vec::new();

        let mut prev: Option<ArtifactId> = None;
        for &b in id_bytes {
            let parents = prev.map(|p| vec![p]).unwrap_or_default();
            let artifact = Artifact {
                id: ArtifactId([b; 32]),
                content: vec![b; 5],
                content_type: "test".into(),
                parent_ids: parents,
                creator: 1,
                created_at: Utc::now(),
                metadata: HashMap::new(),
            };
            let id = artifact.id;
            dag.insert(artifact).unwrap();
            ids.push(id);
            prev = Some(id);
        }
        (dag, ids)
    }

    #[test]
    fn test_create_branch() {
        let (dag, ids) = make_dag_with_artifacts(&[1]);
        let mut mgr = BranchManager::new();
        mgr.create_branch("main", ids[0], &dag).unwrap();
        assert_eq!(mgr.get_head("main"), Some(ids[0]));
    }

    #[test]
    fn test_create_branch_duplicate_fails() {
        let (dag, ids) = make_dag_with_artifacts(&[1]);
        let mut mgr = BranchManager::new();
        mgr.create_branch("main", ids[0], &dag).unwrap();
        let result = mgr.create_branch("main", ids[0], &dag);
        assert!(matches!(result, Err(ArtifactError::BranchConflict(_))));
    }

    #[test]
    fn test_create_branch_missing_head_fails() {
        let dag = ArtifactDag::new();
        let mut mgr = BranchManager::new();
        let result = mgr.create_branch("main", ArtifactId([99; 32]), &dag);
        assert!(matches!(result, Err(ArtifactError::NotFound(_))));
    }

    #[test]
    fn test_advance_branch() {
        let (dag, ids) = make_dag_with_artifacts(&[1, 2]);
        let mut mgr = BranchManager::new();
        mgr.create_branch("main", ids[0], &dag).unwrap();

        let (old, new) = mgr.advance("main", ids[1], &dag).unwrap();
        assert_eq!(old, ids[0]);
        assert_eq!(new, ids[1]);
        assert_eq!(mgr.get_head("main"), Some(ids[1]));
    }

    #[test]
    fn test_advance_nonexistent_branch_fails() {
        let (dag, ids) = make_dag_with_artifacts(&[1]);
        let mut mgr = BranchManager::new();
        let result = mgr.advance("main", ids[0], &dag);
        assert!(matches!(result, Err(ArtifactError::NotFound(_))));
    }

    #[test]
    fn test_advance_missing_head_fails() {
        let (dag, ids) = make_dag_with_artifacts(&[1]);
        let mut mgr = BranchManager::new();
        mgr.create_branch("main", ids[0], &dag).unwrap();
        let result = mgr.advance("main", ArtifactId([99; 32]), &dag);
        assert!(matches!(result, Err(ArtifactError::NotFound(_))));
    }

    #[test]
    fn test_get_head_nonexistent() {
        let mgr = BranchManager::new();
        assert!(mgr.get_head("nope").is_none());
    }

    #[test]
    fn test_list_branches() {
        let (dag, ids) = make_dag_with_artifacts(&[1, 2]);
        let mut mgr = BranchManager::new();
        mgr.create_branch("main", ids[0], &dag).unwrap();
        mgr.create_branch("dev", ids[1], &dag).unwrap();

        let mut branches = mgr.list_branches();
        branches.sort();
        assert_eq!(branches, vec!["dev", "main"]);
    }

    #[test]
    fn test_delete_branch() {
        let (dag, ids) = make_dag_with_artifacts(&[1]);
        let mut mgr = BranchManager::new();
        mgr.create_branch("main", ids[0], &dag).unwrap();

        let removed = mgr.delete_branch("main").unwrap();
        assert_eq!(removed, ids[0]);
        assert!(mgr.get_head("main").is_none());
    }

    #[test]
    fn test_delete_nonexistent_branch_fails() {
        let mut mgr = BranchManager::new();
        let result = mgr.delete_branch("nope");
        assert!(matches!(result, Err(ArtifactError::NotFound(_))));
    }

    #[test]
    fn test_len_and_is_empty() {
        let (dag, ids) = make_dag_with_artifacts(&[1]);
        let mut mgr = BranchManager::new();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);

        mgr.create_branch("main", ids[0], &dag).unwrap();
        assert!(!mgr.is_empty());
        assert_eq!(mgr.len(), 1);
    }

    #[test]
    fn test_multiple_branches_same_head() {
        let (dag, ids) = make_dag_with_artifacts(&[1]);
        let mut mgr = BranchManager::new();
        mgr.create_branch("main", ids[0], &dag).unwrap();
        mgr.create_branch("release", ids[0], &dag).unwrap();

        assert_eq!(mgr.get_head("main"), mgr.get_head("release"));
        assert_eq!(mgr.len(), 2);
    }

    #[test]
    fn test_advance_then_delete() {
        let (dag, ids) = make_dag_with_artifacts(&[1, 2, 3]);
        let mut mgr = BranchManager::new();
        mgr.create_branch("feature", ids[0], &dag).unwrap();
        mgr.advance("feature", ids[1], &dag).unwrap();
        mgr.advance("feature", ids[2], &dag).unwrap();

        let removed = mgr.delete_branch("feature").unwrap();
        assert_eq!(removed, ids[2]);
    }
}
