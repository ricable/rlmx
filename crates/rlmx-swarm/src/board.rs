use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::debug;
use uuid::Uuid;

use crate::types::ClusterId;

/// Maximum number of posts per board.
const MAX_POSTS: usize = 1_000;

/// Maximum number of pinned posts per board.
const MAX_PINS: usize = 25;

/// Unique identifier for a board post.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PostId(pub Uuid);

impl PostId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PostId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for PostId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a coordination board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BoardId(pub Uuid);

impl BoardId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for BoardId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for BoardId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A post on a coordination board.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: PostId,
    pub author: u64,
    pub content: String,
    pub attachments: Vec<Uuid>,
    pub parent_post: Option<PostId>,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub timestamp: DateTime<Utc>,
}

/// A coordination board scoped to a cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    pub id: BoardId,
    pub name: String,
    pub posts: Vec<Post>,
    pub participants: HashSet<u64>,
    pub cluster_id: ClusterId,
}

/// Errors produced by board operations.
#[derive(Debug, thiserror::Error)]
pub enum BoardError {
    #[error("board not found: {0}")]
    BoardNotFound(BoardId),

    #[error("post not found: {0}")]
    PostNotFound(PostId),

    #[error("board full: max {MAX_POSTS} posts")]
    BoardFull,

    #[error("pin limit reached: max {MAX_PINS} pins per board")]
    PinLimitReached,
}

/// Manages coordination boards for inter-agent communication.
#[derive(Debug, Default)]
pub struct BoardManager {
    boards: HashMap<BoardId, Board>,
}

impl BoardManager {
    /// Create a new empty board manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new board for a cluster.
    pub fn create_board(&mut self, name: &str, cluster_id: ClusterId) -> BoardId {
        let id = BoardId::new();
        let board = Board {
            id,
            name: name.to_string(),
            posts: Vec::new(),
            participants: HashSet::new(),
            cluster_id,
        };
        debug!(board_id = %id, name, "created board");
        self.boards.insert(id, board);
        id
    }

    /// Post a message to a board. Enforces 1,000 post limit.
    pub fn post(
        &mut self,
        board_id: BoardId,
        author: u64,
        content: String,
        parent_post: Option<PostId>,
        tags: Vec<String>,
        attachments: Vec<Uuid>,
    ) -> Result<PostId, BoardError> {
        let board = self
            .boards
            .get_mut(&board_id)
            .ok_or(BoardError::BoardNotFound(board_id))?;

        if board.posts.len() >= MAX_POSTS {
            return Err(BoardError::BoardFull);
        }

        // Validate parent post exists if specified
        if let Some(parent_id) = parent_post {
            if !board.posts.iter().any(|p| p.id == parent_id) {
                return Err(BoardError::PostNotFound(parent_id));
            }
        }

        let post_id = PostId::new();
        let post = Post {
            id: post_id,
            author,
            content,
            attachments,
            parent_post,
            tags,
            pinned: false,
            timestamp: Utc::now(),
        };

        board.posts.push(post);
        board.participants.insert(author);
        debug!(board_id = %board_id, post_id = %post_id, "posted to board");

        Ok(post_id)
    }

    /// Pin a post. Enforces 25 pin limit.
    pub fn pin(&mut self, board_id: BoardId, post_id: PostId) -> Result<(), BoardError> {
        let board = self
            .boards
            .get_mut(&board_id)
            .ok_or(BoardError::BoardNotFound(board_id))?;

        // Find post index first, check pin state and count without holding a mutable ref
        let post_idx = board
            .posts
            .iter()
            .position(|p| p.id == post_id)
            .ok_or(BoardError::PostNotFound(post_id))?;

        if board.posts[post_idx].pinned {
            return Ok(()); // Already pinned, no-op
        }

        let pinned_count = board.posts.iter().filter(|p| p.pinned).count();
        if pinned_count >= MAX_PINS {
            return Err(BoardError::PinLimitReached);
        }

        board.posts[post_idx].pinned = true;
        debug!(board_id = %board_id, post_id = %post_id, "pinned post");
        Ok(())
    }

    /// Unpin a post.
    pub fn unpin(&mut self, board_id: BoardId, post_id: PostId) -> Result<(), BoardError> {
        let board = self
            .boards
            .get_mut(&board_id)
            .ok_or(BoardError::BoardNotFound(board_id))?;

        let post = board
            .posts
            .iter_mut()
            .find(|p| p.id == post_id)
            .ok_or(BoardError::PostNotFound(post_id))?;

        post.pinned = false;
        debug!(board_id = %board_id, post_id = %post_id, "unpinned post");
        Ok(())
    }

    /// Get a specific post from a board.
    pub fn get_post(&self, board_id: BoardId, post_id: PostId) -> Option<&Post> {
        self.boards
            .get(&board_id)
            .and_then(|b| b.posts.iter().find(|p| p.id == post_id))
    }

    /// Get a thread: root post + all transitive replies.
    /// Uses a parent→children index and HashSet for O(thread_size) instead of O(N²).
    pub fn get_thread(&self, board_id: BoardId, root_post_id: PostId) -> Vec<&Post> {
        let board = match self.boards.get(&board_id) {
            Some(b) => b,
            None => return Vec::new(),
        };

        let root = match board.posts.iter().find(|p| p.id == root_post_id) {
            Some(r) => r,
            None => return Vec::new(),
        };

        // Build parent→children index once
        let mut children_index: HashMap<PostId, Vec<usize>> = HashMap::new();
        for (idx, post) in board.posts.iter().enumerate() {
            if let Some(parent_id) = post.parent_post {
                children_index.entry(parent_id).or_default().push(idx);
            }
        }

        let mut result = vec![root];
        let mut visited = HashSet::new();
        visited.insert(root_post_id);

        // BFS using the index
        let mut queue = vec![root_post_id];
        while let Some(current_id) = queue.pop() {
            if let Some(child_indices) = children_index.get(&current_id) {
                for &idx in child_indices {
                    let post = &board.posts[idx];
                    if visited.insert(post.id) {
                        result.push(post);
                        queue.push(post.id);
                    }
                }
            }
        }

        result
    }

    /// Query posts by tags (match any of the given tags).
    pub fn query_by_tags(&self, board_id: BoardId, tags: &[String]) -> Vec<&Post> {
        let board = match self.boards.get(&board_id) {
            Some(b) => b,
            None => return Vec::new(),
        };

        let tag_set: HashSet<&str> = tags.iter().map(|s| s.as_str()).collect();
        board
            .posts
            .iter()
            .filter(|p| p.tags.iter().any(|t| tag_set.contains(t.as_str())))
            .collect()
    }

    /// List all boards.
    pub fn list_boards(&self) -> Vec<&Board> {
        self.boards.values().collect()
    }

    /// Get a board by ID.
    pub fn get_board(&self, board_id: BoardId) -> Option<&Board> {
        self.boards.get(&board_id)
    }

    /// Delete a board.
    pub fn delete_board(&mut self, board_id: BoardId) -> Result<(), BoardError> {
        self.boards
            .remove(&board_id)
            .map(|_| ())
            .ok_or(BoardError::BoardNotFound(board_id))
    }

    /// Number of boards.
    pub fn len(&self) -> usize {
        self.boards.len()
    }

    /// Whether there are no boards.
    pub fn is_empty(&self) -> bool {
        self.boards.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cluster() -> ClusterId {
        ClusterId::new()
    }

    #[test]
    fn test_create_board() {
        let mut mgr = BoardManager::new();
        let id = mgr.create_board("findings", cluster());
        let board = mgr.get_board(id).unwrap();
        assert_eq!(board.name, "findings");
        assert!(board.posts.is_empty());
    }

    #[test]
    fn test_post_to_board() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("test", cluster());
        let pid = mgr
            .post(bid, 1, "Hello agents!".into(), None, vec!["greeting".into()], vec![])
            .unwrap();
        let post = mgr.get_post(bid, pid).unwrap();
        assert_eq!(post.content, "Hello agents!");
        assert_eq!(post.author, 1);
        assert_eq!(post.tags, vec!["greeting"]);
    }

    #[test]
    fn test_post_to_nonexistent_board() {
        let mut mgr = BoardManager::new();
        let result = mgr.post(BoardId::new(), 1, "x".into(), None, vec![], vec![]);
        assert!(matches!(result, Err(BoardError::BoardNotFound(_))));
    }

    #[test]
    fn test_post_limit() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("full", cluster());
        for i in 0..MAX_POSTS {
            mgr.post(bid, 1, format!("post-{i}"), None, vec![], vec![])
                .unwrap();
        }
        let result = mgr.post(bid, 1, "overflow".into(), None, vec![], vec![]);
        assert!(matches!(result, Err(BoardError::BoardFull)));
    }

    #[test]
    fn test_threading() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("discussion", cluster());
        let root = mgr.post(bid, 1, "Topic".into(), None, vec![], vec![]).unwrap();
        let r1 = mgr
            .post(bid, 2, "Reply 1".into(), Some(root), vec![], vec![])
            .unwrap();
        mgr.post(bid, 3, "Reply 2".into(), Some(root), vec![], vec![])
            .unwrap();
        mgr.post(bid, 4, "Nested".into(), Some(r1), vec![], vec![])
            .unwrap();

        let thread = mgr.get_thread(bid, root);
        assert_eq!(thread.len(), 4);
        assert_eq!(thread[0].id, root);
    }

    #[test]
    fn test_thread_nonexistent_board() {
        let mgr = BoardManager::new();
        let thread = mgr.get_thread(BoardId::new(), PostId::new());
        assert!(thread.is_empty());
    }

    #[test]
    fn test_thread_nonexistent_root() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("test", cluster());
        let thread = mgr.get_thread(bid, PostId::new());
        assert!(thread.is_empty());
    }

    #[test]
    fn test_pin() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("pins", cluster());
        let pid = mgr.post(bid, 1, "Important".into(), None, vec![], vec![]).unwrap();
        mgr.pin(bid, pid).unwrap();
        assert!(mgr.get_post(bid, pid).unwrap().pinned);
    }

    #[test]
    fn test_pin_idempotent() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("pins", cluster());
        let pid = mgr.post(bid, 1, "x".into(), None, vec![], vec![]).unwrap();
        mgr.pin(bid, pid).unwrap();
        mgr.pin(bid, pid).unwrap(); // should not error
        assert!(mgr.get_post(bid, pid).unwrap().pinned);
    }

    #[test]
    fn test_unpin() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("pins", cluster());
        let pid = mgr.post(bid, 1, "x".into(), None, vec![], vec![]).unwrap();
        mgr.pin(bid, pid).unwrap();
        mgr.unpin(bid, pid).unwrap();
        assert!(!mgr.get_post(bid, pid).unwrap().pinned);
    }

    #[test]
    fn test_pin_limit() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("pins", cluster());
        let mut pids = Vec::new();
        for i in 0..26 {
            pids.push(
                mgr.post(bid, 1, format!("post-{i}"), None, vec![], vec![])
                    .unwrap(),
            );
        }
        for pid in &pids[..MAX_PINS] {
            mgr.pin(bid, *pid).unwrap();
        }
        let result = mgr.pin(bid, pids[25]);
        assert!(matches!(result, Err(BoardError::PinLimitReached)));
    }

    #[test]
    fn test_pin_nonexistent_board() {
        let mut mgr = BoardManager::new();
        let result = mgr.pin(BoardId::new(), PostId::new());
        assert!(matches!(result, Err(BoardError::BoardNotFound(_))));
    }

    #[test]
    fn test_pin_nonexistent_post() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("test", cluster());
        let result = mgr.pin(bid, PostId::new());
        assert!(matches!(result, Err(BoardError::PostNotFound(_))));
    }

    #[test]
    fn test_query_by_tags() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("tagged", cluster());
        mgr.post(bid, 1, "A".into(), None, vec!["urgent".into(), "review".into()], vec![])
            .unwrap();
        mgr.post(bid, 2, "B".into(), None, vec!["review".into()], vec![])
            .unwrap();
        mgr.post(bid, 3, "C".into(), None, vec!["info".into()], vec![])
            .unwrap();

        let urgent = mgr.query_by_tags(bid, &["urgent".into()]);
        assert_eq!(urgent.len(), 1);
        assert_eq!(urgent[0].content, "A");

        let review = mgr.query_by_tags(bid, &["review".into()]);
        assert_eq!(review.len(), 2);
    }

    #[test]
    fn test_query_by_tags_nonexistent_board() {
        let mgr = BoardManager::new();
        let results = mgr.query_by_tags(BoardId::new(), &["tag".into()]);
        assert!(results.is_empty());
    }

    #[test]
    fn test_list_boards() {
        let mut mgr = BoardManager::new();
        let c = cluster();
        mgr.create_board("board1", c);
        mgr.create_board("board2", c);
        assert_eq!(mgr.list_boards().len(), 2);
    }

    #[test]
    fn test_delete_board() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("temp", cluster());
        mgr.delete_board(bid).unwrap();
        assert!(mgr.get_board(bid).is_none());
    }

    #[test]
    fn test_delete_nonexistent_board() {
        let mut mgr = BoardManager::new();
        let result = mgr.delete_board(BoardId::new());
        assert!(matches!(result, Err(BoardError::BoardNotFound(_))));
    }

    #[test]
    fn test_participants_tracked() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("collab", cluster());
        mgr.post(bid, 1, "a".into(), None, vec![], vec![]).unwrap();
        mgr.post(bid, 2, "b".into(), None, vec![], vec![]).unwrap();
        mgr.post(bid, 1, "c".into(), None, vec![], vec![]).unwrap();

        let board = mgr.get_board(bid).unwrap();
        assert_eq!(board.participants.len(), 2);
        assert!(board.participants.contains(&1));
        assert!(board.participants.contains(&2));
    }

    #[test]
    fn test_post_with_attachments() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("artifacts", cluster());
        let att1 = Uuid::new_v4();
        let att2 = Uuid::new_v4();
        let pid = mgr
            .post(bid, 1, "See attached".into(), None, vec![], vec![att1, att2])
            .unwrap();
        let post = mgr.get_post(bid, pid).unwrap();
        assert_eq!(post.attachments.len(), 2);
        assert!(post.attachments.contains(&att1));
    }

    #[test]
    fn test_len_and_is_empty() {
        let mut mgr = BoardManager::new();
        assert!(mgr.is_empty());
        assert_eq!(mgr.len(), 0);
        mgr.create_board("test", cluster());
        assert!(!mgr.is_empty());
        assert_eq!(mgr.len(), 1);
    }

    #[test]
    fn test_post_invalid_parent_post() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("test", cluster());
        let fake_parent = PostId::new();
        let result = mgr.post(bid, 1, "reply".into(), Some(fake_parent), vec![], vec![]);
        assert!(matches!(result, Err(BoardError::PostNotFound(_))));
    }

    #[test]
    fn test_board_id_display() {
        let id = BoardId::new();
        let s = format!("{id}");
        assert!(!s.is_empty());
    }

    #[test]
    fn test_post_id_display() {
        let id = PostId::new();
        let s = format!("{id}");
        assert!(!s.is_empty());
    }

    #[test]
    fn test_board_serialize() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("ser-test", cluster());
        mgr.post(bid, 1, "data".into(), None, vec!["tag".into()], vec![])
            .unwrap();
        let board = mgr.get_board(bid).unwrap();
        let json = serde_json::to_string(board).unwrap();
        assert!(json.contains("ser-test"));
        assert!(json.contains("data"));
    }

    #[test]
    fn test_get_post_from_nonexistent_board() {
        let mgr = BoardManager::new();
        assert!(mgr.get_post(BoardId::new(), PostId::new()).is_none());
    }

    #[test]
    fn test_unpin_nonexistent_post() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("test", cluster());
        let result = mgr.unpin(bid, PostId::new());
        assert!(matches!(result, Err(BoardError::PostNotFound(_))));
    }

    #[test]
    fn test_unpin_nonexistent_board() {
        let mut mgr = BoardManager::new();
        let result = mgr.unpin(BoardId::new(), PostId::new());
        assert!(matches!(result, Err(BoardError::BoardNotFound(_))));
    }

    #[test]
    fn test_query_by_tags_multi_tag_or_semantics() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("multi-tag", cluster());
        mgr.post(bid, 1, "A".into(), None, vec!["urgent".into()], vec![]).unwrap();
        mgr.post(bid, 2, "B".into(), None, vec!["review".into()], vec![]).unwrap();
        mgr.post(bid, 3, "C".into(), None, vec!["info".into()], vec![]).unwrap();
        mgr.post(bid, 4, "D".into(), None, vec!["urgent".into(), "review".into()], vec![]).unwrap();

        // OR semantics: matches posts with "urgent" OR "review"
        let results = mgr.query_by_tags(bid, &["urgent".into(), "review".into()]);
        assert_eq!(results.len(), 3); // A, B, D
        let contents: Vec<&str> = results.iter().map(|p| p.content.as_str()).collect();
        assert!(contents.contains(&"A"));
        assert!(contents.contains(&"B"));
        assert!(contents.contains(&"D"));
        assert!(!contents.contains(&"C"));
    }

    #[test]
    fn test_board_deserialize_roundtrip() {
        let mut mgr = BoardManager::new();
        let bid = mgr.create_board("roundtrip", cluster());
        mgr.post(bid, 1, "first".into(), None, vec!["tag1".into()], vec![]).unwrap();
        mgr.post(bid, 2, "second".into(), None, vec!["tag2".into()], vec![]).unwrap();

        let board = mgr.get_board(bid).unwrap();
        let json = serde_json::to_string(board).unwrap();
        let deserialized: Board = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, board.id);
        assert_eq!(deserialized.name, "roundtrip");
        assert_eq!(deserialized.posts.len(), 2);
        assert_eq!(deserialized.participants.len(), 2);
        assert!(deserialized.participants.contains(&1));
        assert!(deserialized.participants.contains(&2));
    }
}
