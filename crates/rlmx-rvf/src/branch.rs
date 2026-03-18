//! COW (Copy-on-Write) branching for RVF containers.
//!
//! Allows creating lightweight forks of containers, tracking which segments
//! have been modified, and merging branches back.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::container::RvfContainer;
use crate::segments::{DiffKind, RvfSegment, SegmentDiff};
use crate::RvfError;

/// Manages named branches derived from RVF containers.
#[derive(Debug, Clone, Default)]
pub struct BranchManager {
    /// Map of branch name to branch metadata.
    pub branches: HashMap<String, RvfBranch>,
}

/// A single branch (COW fork) of an RVF container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RvfBranch {
    /// Human-readable branch name.
    pub name: String,
    /// UUID of the parent container this branch was derived from.
    pub parent_id: Uuid,
    /// When the branch was created.
    pub created: DateTime<Utc>,
    /// IDs of segments from the parent that have been modified.
    pub modified_segments: Vec<Uuid>,
    /// Entirely new segments added in this branch.
    pub new_segments: Vec<RvfSegment>,
}

impl BranchManager {
    /// Create a new empty branch manager.
    pub fn new() -> Self {
        Self {
            branches: HashMap::new(),
        }
    }

    /// Create a branch from the given container.
    pub fn create_branch(
        &mut self,
        container: &RvfContainer,
        name: &str,
    ) -> Result<&RvfBranch, RvfError> {
        if self.branches.contains_key(name) {
            return Err(RvfError::Branch(format!(
                "branch '{}' already exists",
                name
            )));
        }

        let branch = RvfBranch {
            name: name.to_string(),
            parent_id: container.manifest.id,
            created: Utc::now(),
            modified_segments: Vec::new(),
            new_segments: Vec::new(),
        };

        self.branches.insert(name.to_string(), branch);
        Ok(self.branches.get(name).unwrap())
    }

    /// Merge a branch's new segments into the target container.
    ///
    /// This is a simple append-based merge: new segments from the branch are
    /// added to the target. Modified segment tracking is recorded but
    /// conflict resolution is left to higher-level logic.
    pub fn merge(
        &mut self,
        branch_name: &str,
        target: &mut RvfContainer,
    ) -> Result<(), RvfError> {
        let branch = self
            .branches
            .get(branch_name)
            .ok_or_else(|| RvfError::Branch(format!("branch '{}' not found", branch_name)))?;

        for seg in &branch.new_segments {
            target.segments.push(seg.clone());
            target.manifest.total_size_bytes += seg.data.len() as u64;
        }
        target.manifest.segment_count = target.segments.len();
        // Invalidate signature since content changed.
        target.signature = None;

        self.branches.remove(branch_name);
        Ok(())
    }

    /// Compute the diff between a branch and its parent container.
    pub fn diff(
        &self,
        branch_name: &str,
        container: &RvfContainer,
    ) -> Result<Vec<SegmentDiff>, RvfError> {
        let branch = self
            .branches
            .get(branch_name)
            .ok_or_else(|| RvfError::Branch(format!("branch '{}' not found", branch_name)))?;

        let mut diffs = Vec::new();

        // Modified segments.
        for &seg_id in &branch.modified_segments {
            diffs.push(SegmentDiff {
                segment_id: seg_id,
                kind: DiffKind::Modified,
            });
        }

        // New segments.
        for seg in &branch.new_segments {
            diffs.push(SegmentDiff {
                segment_id: seg.id,
                kind: DiffKind::Added,
            });
        }

        // Segments in container but not referenced (could detect removals in a
        // more sophisticated implementation).
        let _ = container;

        Ok(diffs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container::RvfContainer;

    #[test]
    fn test_branch_creation() {
        let container = RvfContainer::new("test", serde_json::json!({}));
        let mut mgr = BranchManager::new();

        let branch = mgr.create_branch(&container, "feature-x").unwrap();
        assert_eq!(branch.name, "feature-x");
        assert_eq!(branch.parent_id, container.manifest.id);
        assert!(branch.modified_segments.is_empty());
        assert!(branch.new_segments.is_empty());

        // Duplicate name should fail.
        assert!(mgr.create_branch(&container, "feature-x").is_err());
    }
}
