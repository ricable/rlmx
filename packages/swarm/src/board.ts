// ---------------------------------------------------------------------------
// @aix/swarm — Coordination Board (ADR-031)
// ---------------------------------------------------------------------------

import { generateId } from '@aix/shared';

/** Unique identifier for a board post. */
export type PostId = string;

/** Unique identifier for a coordination board. */
export type BoardId = string;

/** A post on a coordination board. */
export interface Post {
  id: PostId;
  author: number;
  content: string;
  attachments: string[];
  parentPost?: PostId;
  tags: string[];
  pinned: boolean;
  timestamp: string;
}

/** A coordination board scoped to a cluster. */
export interface Board {
  id: BoardId;
  name: string;
  posts: Post[];
  participants: Set<number>;
  clusterId: string;
}

/** Errors from board operations. */
export class BoardError extends Error {
  public readonly code: BoardErrorCode;

  constructor(message: string, code: BoardErrorCode) {
    super(message);
    this.code = code;
    this.name = 'BoardError';
    Object.setPrototypeOf(this, BoardError.prototype);
  }
}

export enum BoardErrorCode {
  BoardNotFound = 'BOARD_NOT_FOUND',
  PostNotFound = 'POST_NOT_FOUND',
  BoardFull = 'BOARD_FULL',
  PinLimitReached = 'PIN_LIMIT_REACHED',
}

const MAX_POSTS = 1_000;
const MAX_PINS = 25;

/** Manages coordination boards for inter-agent communication. */
export class BoardManager {
  private boards = new Map<BoardId, Board>();

  /** Create a new board for a cluster. */
  createBoard(name: string, clusterId: string): BoardId {
    const id = generateId();
    const board: Board = {
      id,
      name,
      posts: [],
      participants: new Set(),
      clusterId,
    };
    this.boards.set(id, board);
    return id;
  }

  /** Post a message to a board. Enforces 1,000 post limit. */
  post(
    boardId: BoardId,
    author: number,
    content: string,
    parentPost?: PostId,
    tags: string[] = [],
    attachments: string[] = [],
  ): PostId {
    const board = this.boards.get(boardId);
    if (!board) {
      throw new BoardError(`board ${boardId} not found`, BoardErrorCode.BoardNotFound);
    }
    if (board.posts.length >= MAX_POSTS) {
      throw new BoardError(
        `board ${boardId} has reached the ${MAX_POSTS} post limit`,
        BoardErrorCode.BoardFull,
      );
    }
    if (parentPost !== undefined) {
      if (!board.posts.some((p) => p.id === parentPost)) {
        throw new BoardError(`parent post ${parentPost} not found`, BoardErrorCode.PostNotFound);
      }
    }

    const id = generateId();
    const post: Post = {
      id,
      author,
      content,
      attachments,
      parentPost,
      tags,
      pinned: false,
      timestamp: new Date().toISOString(),
    };

    board.posts.push(post);
    board.participants.add(author);
    return id;
  }

  /** Pin a post. Enforces 25 pin limit. */
  pin(boardId: BoardId, postId: PostId): void {
    const board = this.boards.get(boardId);
    if (!board) {
      throw new BoardError(`board ${boardId} not found`, BoardErrorCode.BoardNotFound);
    }
    const post = board.posts.find((p) => p.id === postId);
    if (!post) {
      throw new BoardError(`post ${postId} not found`, BoardErrorCode.PostNotFound);
    }
    if (post.pinned) return;

    const pinnedCount = board.posts.filter((p) => p.pinned).length;
    if (pinnedCount >= MAX_PINS) {
      throw new BoardError(
        `board ${boardId} has reached the ${MAX_PINS} pin limit`,
        BoardErrorCode.PinLimitReached,
      );
    }
    post.pinned = true;
  }

  /** Unpin a post. */
  unpin(boardId: BoardId, postId: PostId): void {
    const board = this.boards.get(boardId);
    if (!board) {
      throw new BoardError(`board ${boardId} not found`, BoardErrorCode.BoardNotFound);
    }
    const post = board.posts.find((p) => p.id === postId);
    if (!post) {
      throw new BoardError(`post ${postId} not found`, BoardErrorCode.PostNotFound);
    }
    post.pinned = false;
  }

  /** Get a specific post from a board. */
  getPost(boardId: BoardId, postId: PostId): Post | undefined {
    return this.boards.get(boardId)?.posts.find((p) => p.id === postId);
  }

  /** Get a thread: root post + all transitive replies. */
  getThread(boardId: BoardId, rootPostId: PostId): Post[] {
    const board = this.boards.get(boardId);
    if (!board) return [];

    const root = board.posts.find((p) => p.id === rootPostId);
    if (!root) return [];

    const result: Post[] = [root];
    const queue = [rootPostId];

    while (queue.length > 0) {
      const currentId = queue.shift()!;
      for (const post of board.posts) {
        if (post.parentPost === currentId && !result.includes(post)) {
          result.push(post);
          queue.push(post.id);
        }
      }
    }

    return result;
  }

  /** Query posts by tags (match any). */
  queryByTags(boardId: BoardId, tags: string[]): Post[] {
    const board = this.boards.get(boardId);
    if (!board) return [];

    const tagSet = new Set(tags);
    return board.posts.filter((p) => p.tags.some((t) => tagSet.has(t)));
  }

  /** List all boards. */
  listBoards(): Board[] {
    return [...this.boards.values()];
  }

  /** Get a board by ID. */
  getBoard(boardId: BoardId): Board | undefined {
    return this.boards.get(boardId);
  }

  /** Delete a board. */
  deleteBoard(boardId: BoardId): void {
    if (!this.boards.has(boardId)) {
      throw new BoardError(`board ${boardId} not found`, BoardErrorCode.BoardNotFound);
    }
    this.boards.delete(boardId);
  }
}
