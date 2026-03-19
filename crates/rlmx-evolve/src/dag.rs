//! DAG-based version tracking for evolved functions.

use std::collections::HashMap;

use crate::lifecycle::LifecycleManager;
use crate::types::{FunctionId, FunctionStatus};

/// Maximum lineage traversal depth to prevent unbounded recursion.
const MAX_LINEAGE_DEPTH: usize = 100;

/// Directed acyclic graph tracking parent-child relationships between function versions.
#[derive(Debug, Default)]
pub struct FunctionDag {
    children: HashMap<FunctionId, Vec<FunctionId>>,
    parents: HashMap<FunctionId, Option<FunctionId>>,
}

impl FunctionDag {
    /// Create a new empty DAG.
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            parents: HashMap::new(),
        }
    }

    /// Register a function in the DAG with an optional parent.
    pub fn add(&mut self, id: FunctionId, parent_id: Option<FunctionId>) {
        self.parents.insert(id, parent_id);
        self.children.entry(id).or_default();

        if let Some(pid) = parent_id {
            self.children.entry(pid).or_default().push(id);
        }
    }

    /// Create a new placeholder node forked from a parent.
    /// Returns the new node's FunctionId. The actual EvolvedFunction must be
    /// created separately via LifecycleManager.
    pub fn fork(&mut self, parent_id: FunctionId) -> FunctionId {
        let new_id = FunctionId::new();
        self.add(new_id, Some(parent_id));
        new_id
    }

    /// Trace lineage from a node back to its root.
    /// Returns the path from the given node to the root (inclusive).
    /// Max depth 100, cycle-safe.
    pub fn lineage(&self, id: FunctionId) -> Vec<FunctionId> {
        let mut path = Vec::new();
        let mut current = Some(id);
        let mut depth = 0;

        while let Some(cid) = current {
            if depth >= MAX_LINEAGE_DEPTH {
                break;
            }
            // Cycle detection: if we've already seen this id, stop.
            if path.contains(&cid) {
                break;
            }
            path.push(cid);
            current = self.parents.get(&cid).copied().flatten();
            depth += 1;
        }

        path
    }

    /// Get frontier nodes (leaves with no children), excluding Killed functions.
    /// Optionally filter by function name.
    pub fn leaves(
        &self,
        name_filter: Option<&str>,
        lifecycle: &LifecycleManager,
    ) -> Vec<FunctionId> {
        self.children
            .iter()
            .filter(|(_, kids)| kids.is_empty())
            .filter_map(|(id, _)| {
                let func = lifecycle.get(*id)?;
                if func.status == FunctionStatus::Killed {
                    return None;
                }
                if let Some(name) = name_filter {
                    if func.name != name {
                        return None;
                    }
                }
                Some(*id)
            })
            .collect()
    }

    /// Get direct children of a node.
    pub fn children(&self, id: FunctionId) -> Vec<FunctionId> {
        self.children.get(&id).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::LifecycleManager;

    #[test]
    fn add_root_node() {
        let mut dag = FunctionDag::new();
        let id = FunctionId::new();
        dag.add(id, None);
        assert!(dag.children(id).is_empty());
        assert_eq!(dag.lineage(id), vec![id]);
    }

    #[test]
    fn add_with_parent() {
        let mut dag = FunctionDag::new();
        let parent = FunctionId::new();
        let child = FunctionId::new();
        dag.add(parent, None);
        dag.add(child, Some(parent));
        assert_eq!(dag.children(parent), vec![child]);
        assert_eq!(dag.lineage(child), vec![child, parent]);
    }

    #[test]
    fn fork_creates_child() {
        let mut dag = FunctionDag::new();
        let parent = FunctionId::new();
        dag.add(parent, None);
        let child = dag.fork(parent);
        assert_eq!(dag.children(parent), vec![child]);
        assert_eq!(dag.lineage(child), vec![child, parent]);
    }

    #[test]
    fn lineage_chain() {
        let mut dag = FunctionDag::new();
        let a = FunctionId::new();
        let b = FunctionId::new();
        let c = FunctionId::new();
        dag.add(a, None);
        dag.add(b, Some(a));
        dag.add(c, Some(b));
        let lin = dag.lineage(c);
        assert_eq!(lin, vec![c, b, a]);
    }

    #[test]
    fn lineage_max_depth() {
        let mut dag = FunctionDag::new();
        let mut ids = Vec::new();
        let root = FunctionId::new();
        dag.add(root, None);
        ids.push(root);

        for _ in 0..150 {
            let prev = *ids.last().unwrap();
            let next = FunctionId::new();
            dag.add(next, Some(prev));
            ids.push(next);
        }

        let lin = dag.lineage(*ids.last().unwrap());
        assert_eq!(lin.len(), MAX_LINEAGE_DEPTH);
    }

    #[test]
    fn lineage_unknown_node() {
        let dag = FunctionDag::new();
        let id = FunctionId::new();
        // Node not in DAG — still returns [id] since we start with Some(id)
        // and parents lookup returns None, so just [id]
        assert_eq!(dag.lineage(id), vec![id]);
    }

    #[test]
    fn children_empty_for_leaf() {
        let mut dag = FunctionDag::new();
        let id = FunctionId::new();
        dag.add(id, None);
        assert!(dag.children(id).is_empty());
    }

    #[test]
    fn children_nonexistent() {
        let dag = FunctionDag::new();
        assert!(dag.children(FunctionId::new()).is_empty());
    }

    #[test]
    fn leaves_excludes_killed() {
        let mut dag = FunctionDag::new();
        let mut mgr = LifecycleManager::new();

        let id1 = mgr.create("fn", "code1", "goal", None);
        dag.add(id1, None);

        let id2 = mgr.create("fn", "code2", "goal", Some(id1));
        dag.add(id2, Some(id1));

        // Kill id2
        mgr.transition(id2, FunctionStatus::Killed).unwrap();

        let lvs = dag.leaves(None, &mgr);
        // id1 has children (id2), so it's not a leaf
        // id2 is killed, so excluded
        assert!(lvs.is_empty());
    }

    #[test]
    fn leaves_filters_by_name() {
        let mut dag = FunctionDag::new();
        let mut mgr = LifecycleManager::new();

        let id_a = mgr.create("alpha", "code", "goal", None);
        dag.add(id_a, None);

        let id_b = mgr.create("beta", "code", "goal", None);
        dag.add(id_b, None);

        let alpha_leaves = dag.leaves(Some("alpha"), &mgr);
        assert_eq!(alpha_leaves.len(), 1);
        assert_eq!(alpha_leaves[0], id_a);

        let all_leaves = dag.leaves(None, &mgr);
        assert_eq!(all_leaves.len(), 2);
    }

    #[test]
    fn leaves_non_leaf_excluded() {
        let mut dag = FunctionDag::new();
        let mut mgr = LifecycleManager::new();

        let root = mgr.create("fn", "v1", "goal", None);
        dag.add(root, None);
        let child = mgr.create("fn", "v2", "goal", Some(root));
        dag.add(child, Some(root));

        let lvs = dag.leaves(None, &mgr);
        assert_eq!(lvs.len(), 1);
        assert_eq!(lvs[0], child);
    }

    #[test]
    fn fork_multiple_children() {
        let mut dag = FunctionDag::new();
        let parent = FunctionId::new();
        dag.add(parent, None);
        let c1 = dag.fork(parent);
        let c2 = dag.fork(parent);
        let kids = dag.children(parent);
        assert_eq!(kids.len(), 2);
        assert!(kids.contains(&c1));
        assert!(kids.contains(&c2));
    }
}
