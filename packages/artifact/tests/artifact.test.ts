import { describe, it, expect, beforeEach } from 'vitest';
import {
  ArtifactStore,
  ArtifactError,
  ArtifactErrorCode,
  BoardManager,
} from '../src/index.js';

describe('ArtifactStore', () => {
  let store: ArtifactStore;

  beforeEach(() => {
    store = new ArtifactStore();
  });

  it('should store content and return deterministic ID', () => {
    const content = new TextEncoder().encode('hello world');
    const id = store.store(content, 'text/plain', [], 1);
    // SHA-256 of "hello world"
    expect(id).toBe('b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9');
  });

  it('should return same ID for same content', () => {
    const content1 = new TextEncoder().encode('test data');
    const id1 = store.store(content1, 'text/plain', [], 1);
    // Same content again should throw DuplicateId
    const content2 = new TextEncoder().encode('test data');
    expect(() => store.store(content2, 'text/plain', [], 2)).toThrow(ArtifactError);
  });

  it('should retrieve stored artifact', () => {
    const content = new TextEncoder().encode('payload');
    const id = store.store(content, 'application/json', [], 42, { version: '1' });
    const artifact = store.get(id);
    expect(artifact).toBeDefined();
    expect(artifact!.creator).toBe(42);
    expect(artifact!.contentType).toBe('application/json');
    expect(artifact!.metadata.version).toBe('1');
  });

  it('should return undefined for missing artifact', () => {
    expect(store.get('nonexistent')).toBeUndefined();
  });

  it('should track parent-child relationships', () => {
    const parent = store.store(new TextEncoder().encode('parent'), 'text/plain', [], 1);
    const child = store.store(new TextEncoder().encode('child'), 'text/plain', [parent], 1);

    const childArtifact = store.get(child)!;
    expect(childArtifact.parentIds).toEqual([parent]);
    expect(store.children(parent)).toContain(child);
  });

  it('should reject invalid parent', () => {
    expect(() =>
      store.store(new TextEncoder().encode('orphan'), 'text/plain', ['bad-parent'], 1),
    ).toThrow(ArtifactError);
  });

  it('should track roots', () => {
    const r1 = store.store(new TextEncoder().encode('root1'), 'text/plain', [], 1);
    const r2 = store.store(new TextEncoder().encode('root2'), 'text/plain', [], 1);
    store.store(new TextEncoder().encode('child'), 'text/plain', [r1], 1);

    const roots = store.getRoots();
    expect(roots).toContain(r1);
    expect(roots).toContain(r2);
    expect(roots.length).toBe(2);
  });

  it('should compute lineage back to root', () => {
    const a = store.store(new TextEncoder().encode('a'), 'text/plain', [], 1);
    const b = store.store(new TextEncoder().encode('b'), 'text/plain', [a], 1);
    const c = store.store(new TextEncoder().encode('c'), 'text/plain', [b], 1);

    const lineage = store.lineage(c);
    expect(lineage).toEqual([c, b, a]);
  });

  it('should diff artifacts by size', () => {
    const small = store.store(new TextEncoder().encode('hi'), 'text/plain', [], 1);
    const big = store.store(new TextEncoder().encode('hello world!'), 'text/plain', [], 1);

    const diff = store.diff(small, big);
    expect(diff.addedBytes).toBe(10);
    expect(diff.removedBytes).toBe(0);

    const reverseDiff = store.diff(big, small);
    expect(reverseDiff.addedBytes).toBe(0);
    expect(reverseDiff.removedBytes).toBe(10);
  });

  it('should report size', () => {
    expect(store.size).toBe(0);
    store.store(new TextEncoder().encode('x'), 'text/plain', [], 1);
    expect(store.size).toBe(1);
  });

  it('should check existence with has()', () => {
    const id = store.store(new TextEncoder().encode('exists'), 'text/plain', [], 1);
    expect(store.has(id)).toBe(true);
    expect(store.has('nope')).toBe(false);
  });

  // -- Branch tests --

  it('should create and query branches', () => {
    const id = store.store(new TextEncoder().encode('v1'), 'text/plain', [], 1);
    store.createBranch('main', id);
    expect(store.getBranchHead('main')).toBe(id);
    expect(store.listBranches()).toContain('main');
  });

  it('should reject duplicate branch names', () => {
    const id = store.store(new TextEncoder().encode('v1'), 'text/plain', [], 1);
    store.createBranch('main', id);
    expect(() => store.createBranch('main', id)).toThrow(ArtifactError);
  });

  it('should advance a branch', () => {
    const v1 = store.store(new TextEncoder().encode('v1'), 'text/plain', [], 1);
    const v2 = store.store(new TextEncoder().encode('v2'), 'text/plain', [v1], 1);
    store.createBranch('main', v1);

    const [old, newHead] = store.advanceBranch('main', v2);
    expect(old).toBe(v1);
    expect(newHead).toBe(v2);
    expect(store.getBranchHead('main')).toBe(v2);
  });

  it('should delete a branch', () => {
    const id = store.store(new TextEncoder().encode('data'), 'text/plain', [], 1);
    store.createBranch('temp', id);
    const removed = store.deleteBranch('temp');
    expect(removed).toBe(id);
    expect(store.getBranchHead('temp')).toBeUndefined();
  });

  it('should throw when advancing nonexistent branch', () => {
    const id = store.store(new TextEncoder().encode('x'), 'text/plain', [], 1);
    expect(() => store.advanceBranch('nope', id)).toThrow(ArtifactError);
  });
});

describe('BoardManager', () => {
  let mgr: BoardManager;

  beforeEach(() => {
    mgr = new BoardManager();
  });

  it('should create a board', () => {
    const id = mgr.createBoard('findings', 'cluster-1');
    const board = mgr.getBoard(id);
    expect(board).toBeDefined();
    expect(board!.name).toBe('findings');
    expect(board!.clusterId).toBe('cluster-1');
  });

  it('should post to a board', () => {
    const boardId = mgr.createBoard('test', 'c1');
    const postId = mgr.post(boardId, 1, 'Hello agents!', undefined, ['greeting']);
    const post = mgr.getPost(boardId, postId);
    expect(post).toBeDefined();
    expect(post!.content).toBe('Hello agents!');
    expect(post!.tags).toEqual(['greeting']);
  });

  it('should enforce post limit', () => {
    const boardId = mgr.createBoard('full', 'c1');
    for (let i = 0; i < 1000; i++) {
      mgr.post(boardId, 1, `post-${i}`);
    }
    expect(() => mgr.post(boardId, 1, 'overflow')).toThrow(ArtifactError);
  });

  it('should support threading', () => {
    const boardId = mgr.createBoard('discussion', 'c1');
    const root = mgr.post(boardId, 1, 'Topic');
    const reply1 = mgr.post(boardId, 2, 'Reply 1', root);
    const reply2 = mgr.post(boardId, 3, 'Reply 2', root);
    mgr.post(boardId, 4, 'Nested reply', reply1);

    const thread = mgr.getThread(boardId, root);
    expect(thread.length).toBe(4);
    expect(thread[0].id).toBe(root);
  });

  it('should pin and unpin posts', () => {
    const boardId = mgr.createBoard('pins', 'c1');
    const postId = mgr.post(boardId, 1, 'Important');
    mgr.pin(boardId, postId);

    const post = mgr.getPost(boardId, postId);
    expect(post!.pinned).toBe(true);

    mgr.unpin(boardId, postId);
    expect(mgr.getPost(boardId, postId)!.pinned).toBe(false);
  });

  it('should enforce pin limit', () => {
    const boardId = mgr.createBoard('pins', 'c1');
    const postIds: string[] = [];
    for (let i = 0; i < 26; i++) {
      postIds.push(mgr.post(boardId, 1, `post-${i}`));
    }
    for (let i = 0; i < 25; i++) {
      mgr.pin(boardId, postIds[i]);
    }
    expect(() => mgr.pin(boardId, postIds[25])).toThrow(ArtifactError);
  });

  it('should query by tags', () => {
    const boardId = mgr.createBoard('tagged', 'c1');
    mgr.post(boardId, 1, 'A', undefined, ['urgent', 'review']);
    mgr.post(boardId, 2, 'B', undefined, ['review']);
    mgr.post(boardId, 3, 'C', undefined, ['info']);

    const results = mgr.queryByTags(boardId, ['urgent']);
    expect(results.length).toBe(1);
    expect(results[0].content).toBe('A');

    const reviewPosts = mgr.queryByTags(boardId, ['review']);
    expect(reviewPosts.length).toBe(2);
  });

  it('should list and delete boards', () => {
    const b1 = mgr.createBoard('board1', 'c1');
    mgr.createBoard('board2', 'c1');
    expect(mgr.listBoards().length).toBe(2);

    mgr.deleteBoard(b1);
    expect(mgr.listBoards().length).toBe(1);
  });

  it('should throw when deleting nonexistent board', () => {
    expect(() => mgr.deleteBoard('nope')).toThrow(ArtifactError);
  });

  it('should throw when posting to nonexistent board', () => {
    expect(() => mgr.post('nope', 1, 'hello')).toThrow(ArtifactError);
  });

  it('should reject post referencing nonexistent parent post', () => {
    const boardId = mgr.createBoard('test', 'c1');
    expect(() => mgr.post(boardId, 1, 'reply', 'bad-parent')).toThrow(ArtifactError);
  });

  it('should track participants', () => {
    const boardId = mgr.createBoard('collab', 'c1');
    mgr.post(boardId, 1, 'Hello');
    mgr.post(boardId, 2, 'Hi');
    mgr.post(boardId, 1, 'Again');

    const board = mgr.getBoard(boardId)!;
    expect(board.participants.size).toBe(2);
    expect(board.participants.has(1)).toBe(true);
    expect(board.participants.has(2)).toBe(true);
  });

  it('should return empty thread for nonexistent board', () => {
    expect(mgr.getThread('nope', 'nope')).toEqual([]);
  });

  it('should return empty results for tags query on nonexistent board', () => {
    expect(mgr.queryByTags('nope', ['tag'])).toEqual([]);
  });
});
