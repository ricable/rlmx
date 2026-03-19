use std::collections::{HashMap, HashSet};
use tracing::debug;

use crate::types::{Artifact, ArtifactError, ArtifactId};

/// In-memory directed acyclic graph of artifacts.
#[derive(Debug, Default)]
pub struct ArtifactDag {
    artifacts: HashMap<ArtifactId, Artifact>,
    roots: HashSet<ArtifactId>,
    /// Reverse index: parent -> children
    children_index: HashMap<ArtifactId, Vec<ArtifactId>>,
}

impl ArtifactDag {
    /// Create a new empty DAG.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an artifact into the DAG. Parents must already exist.
    pub fn insert(&mut self, artifact: Artifact) -> Result<(), ArtifactError> {
        if self.artifacts.contains_key(&artifact.id) {
            return Err(ArtifactError::DuplicateId(artifact.id));
        }

        // Validate all parents exist
        for parent_id in &artifact.parent_ids {
            if !self.artifacts.contains_key(parent_id) {
                return Err(ArtifactError::InvalidParent(format!(
                    "parent {} not found in DAG",
                    parent_id
                )));
            }
        }

        let id = artifact.id;
        let is_root = artifact.parent_ids.is_empty();

        // Update children index for each parent
        for parent_id in &artifact.parent_ids {
            self.children_index
                .entry(*parent_id)
                .or_default()
                .push(id);
        }

        if is_root {
            self.roots.insert(id);
        }

        debug!(artifact_id = %id, parents = artifact.parent_ids.len(), "inserted artifact into DAG");
        self.artifacts.insert(id, artifact);

        Ok(())
    }

    /// Get an artifact by ID.
    pub fn get(&self, id: &ArtifactId) -> Option<&Artifact> {
        self.artifacts.get(id)
    }

    /// Get the parent artifacts of an artifact.
    pub fn parents(&self, id: &ArtifactId) -> Vec<&Artifact> {
        self.artifacts
            .get(id)
            .map(|a| {
                a.parent_ids
                    .iter()
                    .filter_map(|pid| self.artifacts.get(pid))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get the children of an artifact (artifacts that have this as a parent).
    pub fn children(&self, id: &ArtifactId) -> Vec<&Artifact> {
        self.children_index
            .get(id)
            .map(|ids| ids.iter().filter_map(|cid| self.artifacts.get(cid)).collect())
            .unwrap_or_default()
    }

    /// Trace lineage of an artifact back to a root following the first parent chain.
    pub fn lineage(&self, id: &ArtifactId) -> Vec<ArtifactId> {
        let mut result = Vec::new();
        let mut current = *id;

        loop {
            result.push(current);
            match self.artifacts.get(&current) {
                Some(artifact) if !artifact.parent_ids.is_empty() => {
                    current = artifact.parent_ids[0];
                }
                _ => break,
            }
        }

        result
    }

    /// Return all root artifact IDs (artifacts with no parents).
    pub fn roots(&self) -> Vec<ArtifactId> {
        self.roots.iter().copied().collect()
    }

    /// Check if the DAG contains an artifact with the given ID.
    pub fn contains(&self, id: &ArtifactId) -> bool {
        self.artifacts.contains_key(id)
    }

    /// Number of artifacts in the DAG.
    pub fn len(&self) -> usize {
        self.artifacts.len()
    }

    /// Whether the DAG is empty.
    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Artifact;
    use chrono::Utc;
    use std::collections::HashMap;

    fn make_artifact(id_byte: u8, parents: Vec<ArtifactId>) -> Artifact {
        Artifact {
            id: ArtifactId([id_byte; 32]),
            content: vec![id_byte; 10],
            content_type: "test/bytes".into(),
            parent_ids: parents,
            creator: 1,
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_new_dag_is_empty() {
        let dag = ArtifactDag::new();
        assert!(dag.is_empty());
        assert_eq!(dag.len(), 0);
    }

    #[test]
    fn test_insert_root() {
        let mut dag = ArtifactDag::new();
        let a = make_artifact(1, vec![]);
        dag.insert(a).unwrap();
        assert_eq!(dag.len(), 1);
        assert!(!dag.is_empty());
    }

    #[test]
    fn test_insert_with_parent() {
        let mut dag = ArtifactDag::new();
        let root = make_artifact(1, vec![]);
        let root_id = root.id;
        dag.insert(root).unwrap();

        let child = make_artifact(2, vec![root_id]);
        dag.insert(child).unwrap();
        assert_eq!(dag.len(), 2);
    }

    #[test]
    fn test_insert_duplicate_fails() {
        let mut dag = ArtifactDag::new();
        let a = make_artifact(1, vec![]);
        dag.insert(a.clone()).unwrap();
        let result = dag.insert(a);
        assert!(matches!(result, Err(ArtifactError::DuplicateId(_))));
    }

    #[test]
    fn test_insert_invalid_parent_fails() {
        let mut dag = ArtifactDag::new();
        let missing_parent = ArtifactId([99; 32]);
        let a = make_artifact(1, vec![missing_parent]);
        let result = dag.insert(a);
        assert!(matches!(result, Err(ArtifactError::InvalidParent(_))));
    }

    #[test]
    fn test_get_existing() {
        let mut dag = ArtifactDag::new();
        let a = make_artifact(1, vec![]);
        let id = a.id;
        dag.insert(a).unwrap();
        assert!(dag.get(&id).is_some());
        assert_eq!(dag.get(&id).unwrap().creator, 1);
    }

    #[test]
    fn test_get_nonexistent() {
        let dag = ArtifactDag::new();
        assert!(dag.get(&ArtifactId([42; 32])).is_none());
    }

    #[test]
    fn test_contains() {
        let mut dag = ArtifactDag::new();
        let a = make_artifact(1, vec![]);
        let id = a.id;
        dag.insert(a).unwrap();
        assert!(dag.contains(&id));
        assert!(!dag.contains(&ArtifactId([99; 32])));
    }

    #[test]
    fn test_roots() {
        let mut dag = ArtifactDag::new();
        let r1 = make_artifact(1, vec![]);
        let r2 = make_artifact(2, vec![]);
        let r1_id = r1.id;
        let r2_id = r2.id;
        dag.insert(r1).unwrap();
        dag.insert(r2).unwrap();

        let roots = dag.roots();
        assert_eq!(roots.len(), 2);
        assert!(roots.contains(&r1_id));
        assert!(roots.contains(&r2_id));
    }

    #[test]
    fn test_parents() {
        let mut dag = ArtifactDag::new();
        let root = make_artifact(1, vec![]);
        let root_id = root.id;
        dag.insert(root).unwrap();

        let child = make_artifact(2, vec![root_id]);
        let child_id = child.id;
        dag.insert(child).unwrap();

        let parents = dag.parents(&child_id);
        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0].id, root_id);
    }

    #[test]
    fn test_parents_of_root_is_empty() {
        let mut dag = ArtifactDag::new();
        let root = make_artifact(1, vec![]);
        let root_id = root.id;
        dag.insert(root).unwrap();
        assert!(dag.parents(&root_id).is_empty());
    }

    #[test]
    fn test_children() {
        let mut dag = ArtifactDag::new();
        let root = make_artifact(1, vec![]);
        let root_id = root.id;
        dag.insert(root).unwrap();

        let c1 = make_artifact(2, vec![root_id]);
        let c2 = make_artifact(3, vec![root_id]);
        let c1_id = c1.id;
        let c2_id = c2.id;
        dag.insert(c1).unwrap();
        dag.insert(c2).unwrap();

        let children = dag.children(&root_id);
        assert_eq!(children.len(), 2);
        let child_ids: Vec<ArtifactId> = children.iter().map(|c| c.id).collect();
        assert!(child_ids.contains(&c1_id));
        assert!(child_ids.contains(&c2_id));
    }

    #[test]
    fn test_children_of_leaf_is_empty() {
        let mut dag = ArtifactDag::new();
        let leaf = make_artifact(1, vec![]);
        let leaf_id = leaf.id;
        dag.insert(leaf).unwrap();
        assert!(dag.children(&leaf_id).is_empty());
    }

    #[test]
    fn test_lineage_single_root() {
        let mut dag = ArtifactDag::new();
        let root = make_artifact(1, vec![]);
        let root_id = root.id;
        dag.insert(root).unwrap();

        let lineage = dag.lineage(&root_id);
        assert_eq!(lineage, vec![root_id]);
    }

    #[test]
    fn test_lineage_chain() {
        let mut dag = ArtifactDag::new();
        let a = make_artifact(1, vec![]);
        let a_id = a.id;
        dag.insert(a).unwrap();

        let b = make_artifact(2, vec![a_id]);
        let b_id = b.id;
        dag.insert(b).unwrap();

        let c = make_artifact(3, vec![b_id]);
        let c_id = c.id;
        dag.insert(c).unwrap();

        let lineage = dag.lineage(&c_id);
        assert_eq!(lineage, vec![c_id, b_id, a_id]);
    }

    #[test]
    fn test_lineage_nonexistent() {
        let dag = ArtifactDag::new();
        let lineage = dag.lineage(&ArtifactId([99; 32]));
        assert_eq!(lineage.len(), 1); // just the id itself
    }

    #[test]
    fn test_multi_parent_insert() {
        let mut dag = ArtifactDag::new();
        let p1 = make_artifact(1, vec![]);
        let p2 = make_artifact(2, vec![]);
        let p1_id = p1.id;
        let p2_id = p2.id;
        dag.insert(p1).unwrap();
        dag.insert(p2).unwrap();

        let merge = make_artifact(3, vec![p1_id, p2_id]);
        dag.insert(merge).unwrap();
        assert_eq!(dag.len(), 3);
    }

    #[test]
    fn test_multi_parent_lineage_follows_first() {
        let mut dag = ArtifactDag::new();
        let p1 = make_artifact(1, vec![]);
        let p2 = make_artifact(2, vec![]);
        let p1_id = p1.id;
        let p2_id = p2.id;
        dag.insert(p1).unwrap();
        dag.insert(p2).unwrap();

        let merge = make_artifact(3, vec![p1_id, p2_id]);
        let merge_id = merge.id;
        dag.insert(merge).unwrap();

        let lineage = dag.lineage(&merge_id);
        assert_eq!(lineage, vec![merge_id, p1_id]);
        assert!(!lineage.contains(&p2_id));
    }

    #[test]
    fn test_child_not_root() {
        let mut dag = ArtifactDag::new();
        let root = make_artifact(1, vec![]);
        let root_id = root.id;
        dag.insert(root).unwrap();

        let child = make_artifact(2, vec![root_id]);
        let child_id = child.id;
        dag.insert(child).unwrap();

        let roots = dag.roots();
        assert!(roots.contains(&root_id));
        assert!(!roots.contains(&child_id));
    }
}
