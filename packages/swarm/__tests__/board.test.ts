import { describe, it, expect, beforeEach } from 'vitest';
import { BoardManager, BoardError, BoardErrorCode } from '../src/board.js';

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
    expect(board!.posts).toEqual([]);
  });

  it('should post to a board', () => {
    const boardId = mgr.createBoard('test', 'c1');
    const postId = mgr.post(boardId, 1, 'Hello!', undefined, ['greeting']);
    const post = mgr.getPost(boardId, postId);
    expect(post).toBeDefined();
    expect(post!.content).toBe('Hello!');
    expect(post!.author).toBe(1);
    expect(post!.tags).toEqual(['greeting']);
    expect(post!.pinned).toBe(false);
  });

  it('should reject post to nonexistent board', () => {
    expect(() => mgr.post('bad-id', 1, 'x')).toThrow(BoardError);
  });

  it('should enforce post limit', () => {
    const boardId = mgr.createBoard('full', 'c1');
    for (let i = 0; i < 1000; i++) {
      mgr.post(boardId, 1, `post-${i}`);
    }
    expect(() => mgr.post(boardId, 1, 'overflow')).toThrow(BoardError);
    try {
      mgr.post(boardId, 1, 'overflow');
    } catch (e) {
      expect((e as BoardError).code).toBe(BoardErrorCode.BoardFull);
    }
  });

  it('should support threading', () => {
    const boardId = mgr.createBoard('discussion', 'c1');
    const root = mgr.post(boardId, 1, 'Topic');
    const r1 = mgr.post(boardId, 2, 'Reply 1', root);
    mgr.post(boardId, 3, 'Reply 2', root);
    mgr.post(boardId, 4, 'Nested', r1);

    const thread = mgr.getThread(boardId, root);
    expect(thread.length).toBe(4);
    expect(thread[0].id).toBe(root);
  });

  it('should reject post with nonexistent parent', () => {
    const boardId = mgr.createBoard('test', 'c1');
    expect(() => mgr.post(boardId, 1, 'reply', 'bad-parent')).toThrow(BoardError);
  });

  it('should pin and unpin', () => {
    const boardId = mgr.createBoard('pins', 'c1');
    const postId = mgr.post(boardId, 1, 'Important');
    mgr.pin(boardId, postId);
    expect(mgr.getPost(boardId, postId)!.pinned).toBe(true);

    mgr.unpin(boardId, postId);
    expect(mgr.getPost(boardId, postId)!.pinned).toBe(false);
  });

  it('should enforce pin limit', () => {
    const boardId = mgr.createBoard('pins', 'c1');
    const ids: string[] = [];
    for (let i = 0; i < 26; i++) {
      ids.push(mgr.post(boardId, 1, `post-${i}`));
    }
    for (let i = 0; i < 25; i++) {
      mgr.pin(boardId, ids[i]);
    }
    expect(() => mgr.pin(boardId, ids[25])).toThrow(BoardError);
  });

  it('should query by tags', () => {
    const boardId = mgr.createBoard('tagged', 'c1');
    mgr.post(boardId, 1, 'A', undefined, ['urgent', 'review']);
    mgr.post(boardId, 2, 'B', undefined, ['review']);
    mgr.post(boardId, 3, 'C', undefined, ['info']);

    expect(mgr.queryByTags(boardId, ['urgent']).length).toBe(1);
    expect(mgr.queryByTags(boardId, ['review']).length).toBe(2);
    expect(mgr.queryByTags(boardId, ['missing']).length).toBe(0);
  });

  it('should list and delete boards', () => {
    const b1 = mgr.createBoard('a', 'c1');
    mgr.createBoard('b', 'c1');
    expect(mgr.listBoards().length).toBe(2);

    mgr.deleteBoard(b1);
    expect(mgr.listBoards().length).toBe(1);
  });

  it('should throw when deleting nonexistent board', () => {
    expect(() => mgr.deleteBoard('nope')).toThrow(BoardError);
  });

  it('should track participants', () => {
    const boardId = mgr.createBoard('collab', 'c1');
    mgr.post(boardId, 1, 'a');
    mgr.post(boardId, 2, 'b');
    mgr.post(boardId, 1, 'c');

    const board = mgr.getBoard(boardId)!;
    expect(board.participants.size).toBe(2);
  });

  it('should return empty thread for nonexistent board', () => {
    expect(mgr.getThread('nope', 'nope')).toEqual([]);
  });

  it('should return empty tags query for nonexistent board', () => {
    expect(mgr.queryByTags('nope', ['tag'])).toEqual([]);
  });

  it('should support attachments', () => {
    const boardId = mgr.createBoard('attachments', 'c1');
    const postId = mgr.post(boardId, 1, 'see attached', undefined, [], ['artifact-1', 'artifact-2']);
    const post = mgr.getPost(boardId, postId)!;
    expect(post.attachments).toEqual(['artifact-1', 'artifact-2']);
  });
});
