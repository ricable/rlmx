use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tracing::debug;

use crate::dag::ArtifactDag;
use crate::types::{Artifact, ArtifactDiff, ArtifactError, ArtifactId};

/// Content-addressed artifact store backed by an `ArtifactDag`.
#[derive(Debug, Default)]
pub struct ContentAddressedStore {
    dag: ArtifactDag,
}

impl ContentAddressedStore {
    /// Create a new empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Compute the SHA-256 hash of content bytes.
    fn compute_id(content: &[u8]) -> ArtifactId {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let result = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        ArtifactId(bytes)
    }

    /// Store content and return its content-addressed ID.
    pub fn store(
        &mut self,
        content: &[u8],
        content_type: &str,
        parent_ids: Vec<ArtifactId>,
        creator: u64,
        metadata: HashMap<String, String>,
    ) -> Result<ArtifactId, ArtifactError> {
        let id = Self::compute_id(content);

        let artifact = Artifact {
            id,
            content: content.to_vec(),
            content_type: content_type.to_string(),
            parent_ids,
            creator,
            created_at: chrono::Utc::now(),
            metadata,
        };

        debug!(artifact_id = %id, content_type, size = content.len(), "storing artifact");
        self.dag.insert(artifact)?;

        Ok(id)
    }

    /// Get an artifact by ID.
    pub fn get(&self, id: &ArtifactId) -> Option<&Artifact> {
        self.dag.get(id)
    }

    /// Compute a byte-level diff between two artifacts.
    pub fn diff(&self, a: &ArtifactId, b: &ArtifactId) -> Result<ArtifactDiff, ArtifactError> {
        let art_a = self
            .dag
            .get(a)
            .ok_or_else(|| ArtifactError::NotFound(a.to_string()))?;
        let art_b = self
            .dag
            .get(b)
            .ok_or_else(|| ArtifactError::NotFound(b.to_string()))?;

        let old_len = art_a.content.len();
        let new_len = art_b.content.len();

        let (added_bytes, removed_bytes) = if new_len >= old_len {
            (new_len - old_len, 0)
        } else {
            (0, old_len - new_len)
        };

        Ok(ArtifactDiff {
            old_id: *a,
            new_id: *b,
            content_type: art_b.content_type.clone(),
            added_bytes,
            removed_bytes,
        })
    }

    /// Trace lineage from an artifact back to its root.
    pub fn lineage(&self, id: &ArtifactId) -> Vec<ArtifactId> {
        self.dag.lineage(id)
    }

    /// Get all root artifact IDs.
    pub fn roots(&self) -> Vec<ArtifactId> {
        self.dag.roots()
    }

    /// Check if an artifact exists in the store.
    pub fn contains(&self, id: &ArtifactId) -> bool {
        self.dag.contains(id)
    }

    /// Number of artifacts in the store.
    pub fn len(&self) -> usize {
        self.dag.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.dag.is_empty()
    }

    /// Get the parent artifacts of an artifact.
    pub fn parents(&self, id: &ArtifactId) -> Vec<&Artifact> {
        self.dag.parents(id)
    }

    /// Get the children of an artifact.
    pub fn children(&self, id: &ArtifactId) -> Vec<&Artifact> {
        self.dag.children(id)
    }

    /// Access the underlying DAG (for branch validation).
    pub fn dag(&self) -> &ArtifactDag {
        &self.dag
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_returns_deterministic_id() {
        let mut store = ContentAddressedStore::new();
        let id = store
            .store(b"hello world", "text/plain", vec![], 1, HashMap::new())
            .unwrap();

        // SHA-256 of "hello world" is well-known
        let expected_hex = "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";
        assert_eq!(id.to_hex(), expected_hex);
    }

    #[test]
    fn test_store_same_content_is_duplicate() {
        let mut store = ContentAddressedStore::new();
        store
            .store(b"data", "text/plain", vec![], 1, HashMap::new())
            .unwrap();
        let result = store.store(b"data", "text/plain", vec![], 2, HashMap::new());
        assert!(matches!(result, Err(ArtifactError::DuplicateId(_))));
    }

    #[test]
    fn test_store_different_content_different_id() {
        let mut store = ContentAddressedStore::new();
        let id1 = store
            .store(b"alpha", "text/plain", vec![], 1, HashMap::new())
            .unwrap();
        let id2 = store
            .store(b"beta", "text/plain", vec![], 1, HashMap::new())
            .unwrap();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_store_get() {
        let mut store = ContentAddressedStore::new();
        let id = store
            .store(b"content", "text/plain", vec![], 42, HashMap::new())
            .unwrap();
        let artifact = store.get(&id).unwrap();
        assert_eq!(artifact.content, b"content");
        assert_eq!(artifact.creator, 42);
        assert_eq!(artifact.content_type, "text/plain");
    }

    #[test]
    fn test_store_get_nonexistent() {
        let store = ContentAddressedStore::new();
        assert!(store.get(&ArtifactId([0; 32])).is_none());
    }

    #[test]
    fn test_store_with_parents() {
        let mut store = ContentAddressedStore::new();
        let parent = store
            .store(b"parent", "text/plain", vec![], 1, HashMap::new())
            .unwrap();
        let child = store
            .store(b"child", "text/plain", vec![parent], 1, HashMap::new())
            .unwrap();

        let artifact = store.get(&child).unwrap();
        assert_eq!(artifact.parent_ids, vec![parent]);
    }

    #[test]
    fn test_store_invalid_parent() {
        let mut store = ContentAddressedStore::new();
        let fake_parent = ArtifactId([99; 32]);
        let result = store.store(b"orphan", "text/plain", vec![fake_parent], 1, HashMap::new());
        assert!(matches!(result, Err(ArtifactError::InvalidParent(_))));
    }

    #[test]
    fn test_store_with_metadata() {
        let mut store = ContentAddressedStore::new();
        let mut meta = HashMap::new();
        meta.insert("author".into(), "agent-7".into());
        meta.insert("version".into(), "1.0".into());

        let id = store
            .store(b"data", "application/json", vec![], 7, meta)
            .unwrap();
        let artifact = store.get(&id).unwrap();
        assert_eq!(artifact.metadata.get("author").unwrap(), "agent-7");
        assert_eq!(artifact.metadata.get("version").unwrap(), "1.0");
    }

    #[test]
    fn test_diff_growth() {
        let mut store = ContentAddressedStore::new();
        let a = store
            .store(b"short", "text/plain", vec![], 1, HashMap::new())
            .unwrap();
        let b = store
            .store(b"much longer content", "text/plain", vec![], 1, HashMap::new())
            .unwrap();

        let diff = store.diff(&a, &b).unwrap();
        assert_eq!(diff.old_id, a);
        assert_eq!(diff.new_id, b);
        assert!(diff.added_bytes > 0);
        assert_eq!(diff.removed_bytes, 0);
    }

    #[test]
    fn test_diff_shrink() {
        let mut store = ContentAddressedStore::new();
        let a = store
            .store(b"long content here", "text/plain", vec![], 1, HashMap::new())
            .unwrap();
        let b = store
            .store(b"tiny", "text/plain", vec![], 1, HashMap::new())
            .unwrap();

        let diff = store.diff(&a, &b).unwrap();
        assert_eq!(diff.added_bytes, 0);
        assert!(diff.removed_bytes > 0);
    }

    #[test]
    fn test_diff_same_size() {
        let mut store = ContentAddressedStore::new();
        let a = store
            .store(b"aaaa", "text/plain", vec![], 1, HashMap::new())
            .unwrap();
        let b = store
            .store(b"bbbb", "text/plain", vec![], 1, HashMap::new())
            .unwrap();

        let diff = store.diff(&a, &b).unwrap();
        assert_eq!(diff.added_bytes, 0);
        assert_eq!(diff.removed_bytes, 0);
    }

    #[test]
    fn test_diff_not_found() {
        let mut store = ContentAddressedStore::new();
        let a = store
            .store(b"exists", "text/plain", vec![], 1, HashMap::new())
            .unwrap();
        let missing = ArtifactId([0; 32]);

        assert!(store.diff(&a, &missing).is_err());
        assert!(store.diff(&missing, &a).is_err());
    }

    #[test]
    fn test_lineage_through_store() {
        let mut store = ContentAddressedStore::new();
        let a = store
            .store(b"root", "text/plain", vec![], 1, HashMap::new())
            .unwrap();
        let b = store
            .store(b"child", "text/plain", vec![a], 1, HashMap::new())
            .unwrap();
        let c = store
            .store(b"grandchild", "text/plain", vec![b], 1, HashMap::new())
            .unwrap();

        let lineage = store.lineage(&c);
        assert_eq!(lineage, vec![c, b, a]);
    }

    #[test]
    fn test_store_roots() {
        let mut store = ContentAddressedStore::new();
        let r1 = store
            .store(b"root1", "text/plain", vec![], 1, HashMap::new())
            .unwrap();
        let r2 = store
            .store(b"root2", "text/plain", vec![], 1, HashMap::new())
            .unwrap();
        store
            .store(b"child", "text/plain", vec![r1], 1, HashMap::new())
            .unwrap();

        let roots = store.roots();
        assert_eq!(roots.len(), 2);
        assert!(roots.contains(&r1));
        assert!(roots.contains(&r2));
    }

    #[test]
    fn test_store_contains() {
        let mut store = ContentAddressedStore::new();
        let id = store
            .store(b"test", "text/plain", vec![], 1, HashMap::new())
            .unwrap();
        assert!(store.contains(&id));
        assert!(!store.contains(&ArtifactId([0; 32])));
    }

    #[test]
    fn test_store_len() {
        let mut store = ContentAddressedStore::new();
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());

        store
            .store(b"one", "text/plain", vec![], 1, HashMap::new())
            .unwrap();
        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());
    }

    #[test]
    fn test_store_parents_and_children() {
        let mut store = ContentAddressedStore::new();
        let parent = store
            .store(b"parent", "text/plain", vec![], 1, HashMap::new())
            .unwrap();
        let child = store
            .store(b"child", "text/plain", vec![parent], 1, HashMap::new())
            .unwrap();

        assert_eq!(store.parents(&child).len(), 1);
        assert_eq!(store.parents(&child)[0].id, parent);
        assert_eq!(store.children(&parent).len(), 1);
        assert_eq!(store.children(&parent)[0].id, child);
    }
}
