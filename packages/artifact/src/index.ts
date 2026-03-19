// ---------------------------------------------------------------------------
// @aix/artifact — Content-addressed artifact DAG for the @aix ecosystem
// ---------------------------------------------------------------------------

// Use crypto.randomUUID() directly (same as @aix/shared generateId)
// to avoid build-order dependency on @aix/shared dist
function generateId(): string {
  return crypto.randomUUID();
}

/** Content-addressed artifact ID (hex-encoded SHA-256 hash). */
export type ArtifactId = string;

/** A content-addressed artifact in the DAG. */
export interface Artifact {
  id: ArtifactId;
  content: Uint8Array;
  contentType: string;
  parentIds: ArtifactId[];
  creator: number;
  createdAt: string; // ISO 8601
  metadata: Record<string, string>;
}

/** Diff between two artifacts. */
export interface ArtifactDiff {
  oldId: ArtifactId;
  newId: ArtifactId;
  contentType: string;
  addedBytes: number;
  removedBytes: number;
}

/** Error codes for artifact operations. */
export enum ArtifactErrorCode {
  NotFound = 'NOT_FOUND',
  DuplicateId = 'DUPLICATE_ID',
  InvalidParent = 'INVALID_PARENT',
  BranchConflict = 'BRANCH_CONFLICT',
  StoreError = 'STORE_ERROR',
  BoardFull = 'BOARD_FULL',
  PinLimitReached = 'PIN_LIMIT_REACHED',
  PostNotFound = 'POST_NOT_FOUND',
  BoardNotFound = 'BOARD_NOT_FOUND',
}

/** Artifact operation error. */
export class ArtifactError extends Error {
  public readonly code: ArtifactErrorCode;

  constructor(message: string, code: ArtifactErrorCode) {
    super(message);
    this.code = code;
    this.name = 'ArtifactError';
    Object.setPrototypeOf(this, ArtifactError.prototype);
  }
}

// ---------------------------------------------------------------------------
// SHA-256 helper (Web Crypto API compatible, sync fallback for Node)
// ---------------------------------------------------------------------------

function sha256Hex(data: Uint8Array): string {
  // Use Node.js crypto for synchronous hashing
  try {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const crypto = require('crypto');
    const hash = crypto.createHash('sha256');
    hash.update(data);
    return hash.digest('hex');
  } catch {
    // Fallback: simple hash for environments without crypto
    // This should not happen in Node.js
    let hash = 0;
    for (let i = 0; i < data.length; i++) {
      hash = ((hash << 5) - hash + data[i]) | 0;
    }
    return Math.abs(hash).toString(16).padStart(64, '0');
  }
}

// ---------------------------------------------------------------------------
// ArtifactStore — In-memory content-addressed store with DAG + branches
// ---------------------------------------------------------------------------

/** In-memory content-addressed artifact store with DAG structure and branches. */
export class ArtifactStore {
  private artifacts = new Map<ArtifactId, Artifact>();
  private childrenIndex = new Map<ArtifactId, ArtifactId[]>();
  private roots = new Set<ArtifactId>();
  private branches = new Map<string, ArtifactId>();

  /** Store content and return its content-addressed ID. */
  store(
    content: Uint8Array,
    contentType: string,
    parentIds: ArtifactId[],
    creator: number,
    metadata: Record<string, string> = {},
  ): ArtifactId {
    const id = sha256Hex(content);

    if (this.artifacts.has(id)) {
      throw new ArtifactError(
        `duplicate artifact id: ${id}`,
        ArtifactErrorCode.DuplicateId,
      );
    }

    for (const pid of parentIds) {
      if (!this.artifacts.has(pid)) {
        throw new ArtifactError(
          `parent ${pid} not found`,
          ArtifactErrorCode.InvalidParent,
        );
      }
    }

    const artifact: Artifact = {
      id,
      content,
      contentType,
      parentIds,
      creator,
      createdAt: new Date().toISOString(),
      metadata,
    };

    this.artifacts.set(id, artifact);

    if (parentIds.length === 0) {
      this.roots.add(id);
    }

    for (const pid of parentIds) {
      const children = this.childrenIndex.get(pid) ?? [];
      children.push(id);
      this.childrenIndex.set(pid, children);
    }

    return id;
  }

  /** Get an artifact by ID. */
  get(id: ArtifactId): Artifact | undefined {
    return this.artifacts.get(id);
  }

  /** Compute a byte-level diff between two artifacts. */
  diff(a: ArtifactId, b: ArtifactId): ArtifactDiff {
    const artA = this.artifacts.get(a);
    if (!artA) {
      throw new ArtifactError(`artifact ${a} not found`, ArtifactErrorCode.NotFound);
    }
    const artB = this.artifacts.get(b);
    if (!artB) {
      throw new ArtifactError(`artifact ${b} not found`, ArtifactErrorCode.NotFound);
    }

    const oldLen = artA.content.length;
    const newLen = artB.content.length;

    return {
      oldId: a,
      newId: b,
      contentType: artB.contentType,
      addedBytes: newLen > oldLen ? newLen - oldLen : 0,
      removedBytes: oldLen > newLen ? oldLen - newLen : 0,
    };
  }

  /** Trace lineage from an artifact back to a root via the first parent chain. */
  lineage(id: ArtifactId): ArtifactId[] {
    const result: ArtifactId[] = [];
    let current: ArtifactId | undefined = id;

    while (current !== undefined) {
      result.push(current);
      const artifact = this.artifacts.get(current);
      if (!artifact || artifact.parentIds.length === 0) break;
      current = artifact.parentIds[0];
    }

    return result;
  }

  /** Get all root artifact IDs. */
  getRoots(): ArtifactId[] {
    return [...this.roots];
  }

  /** Get children of an artifact. */
  children(id: ArtifactId): ArtifactId[] {
    return this.childrenIndex.get(id) ?? [];
  }

  /** Check if an artifact exists. */
  has(id: ArtifactId): boolean {
    return this.artifacts.has(id);
  }

  /** Number of artifacts. */
  get size(): number {
    return this.artifacts.size;
  }

  // -- Branch operations ---------------------------------------------------

  /** Create a named branch pointing at the given head. */
  createBranch(name: string, headId: ArtifactId): void {
    if (this.branches.has(name)) {
      throw new ArtifactError(
        `branch '${name}' already exists`,
        ArtifactErrorCode.BranchConflict,
      );
    }
    if (!this.artifacts.has(headId)) {
      throw new ArtifactError(
        `head artifact ${headId} not found`,
        ArtifactErrorCode.NotFound,
      );
    }
    this.branches.set(name, headId);
  }

  /** Advance a branch to a new head. Returns [oldHead, newHead]. */
  advanceBranch(name: string, newHeadId: ArtifactId): [ArtifactId, ArtifactId] {
    const oldHead = this.branches.get(name);
    if (oldHead === undefined) {
      throw new ArtifactError(
        `branch '${name}' not found`,
        ArtifactErrorCode.NotFound,
      );
    }
    if (!this.artifacts.has(newHeadId)) {
      throw new ArtifactError(
        `new head artifact ${newHeadId} not found`,
        ArtifactErrorCode.NotFound,
      );
    }
    this.branches.set(name, newHeadId);
    return [oldHead, newHeadId];
  }

  /** Get the head artifact ID for a branch. */
  getBranchHead(name: string): ArtifactId | undefined {
    return this.branches.get(name);
  }

  /** List all branch names. */
  listBranches(): string[] {
    return [...this.branches.keys()];
  }

  /** Delete a branch. Returns the removed head ID. */
  deleteBranch(name: string): ArtifactId {
    const head = this.branches.get(name);
    if (head === undefined) {
      throw new ArtifactError(
        `branch '${name}' not found`,
        ArtifactErrorCode.NotFound,
      );
    }
    this.branches.delete(name);
    return head;
  }
}

// ---------------------------------------------------------------------------
// Board types (coordination board for inter-agent communication, ADR-031)
// ---------------------------------------------------------------------------

/** Unique identifier for a board post. */
export type PostId = string;

/** Unique identifier for a coordination board. */
export type BoardId = string;

/** A post on a coordination board. */
export interface Post {
  id: PostId;
  author: number;
  content: string;
  attachments: string[]; // artifact IDs
  parentPost?: PostId;
  tags: string[];
  pinned: boolean;
  timestamp: string; // ISO 8601
}

/** A coordination board scoped to a cluster. */
export interface Board {
  id: BoardId;
  name: string;
  posts: Post[];
  participants: Set<number>;
  clusterId: string;
}

const MAX_POSTS_PER_BOARD = 1_000;
const MAX_PINS_PER_BOARD = 25;

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
      throw new ArtifactError(
        `board ${boardId} not found`,
        ArtifactErrorCode.BoardNotFound,
      );
    }
    if (board.posts.length >= MAX_POSTS_PER_BOARD) {
      throw new ArtifactError(
        `board ${boardId} has reached the ${MAX_POSTS_PER_BOARD} post limit`,
        ArtifactErrorCode.BoardFull,
      );
    }
    if (parentPost !== undefined) {
      const parentExists = board.posts.some((p) => p.id === parentPost);
      if (!parentExists) {
        throw new ArtifactError(
          `parent post ${parentPost} not found`,
          ArtifactErrorCode.PostNotFound,
        );
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
      throw new ArtifactError(`board ${boardId} not found`, ArtifactErrorCode.BoardNotFound);
    }
    const post = board.posts.find((p) => p.id === postId);
    if (!post) {
      throw new ArtifactError(`post ${postId} not found`, ArtifactErrorCode.PostNotFound);
    }
    if (post.pinned) return;

    const pinnedCount = board.posts.filter((p) => p.pinned).length;
    if (pinnedCount >= MAX_PINS_PER_BOARD) {
      throw new ArtifactError(
        `board ${boardId} has reached the ${MAX_PINS_PER_BOARD} pin limit`,
        ArtifactErrorCode.PinLimitReached,
      );
    }
    post.pinned = true;
  }

  /** Unpin a post. */
  unpin(boardId: BoardId, postId: PostId): void {
    const board = this.boards.get(boardId);
    if (!board) {
      throw new ArtifactError(`board ${boardId} not found`, ArtifactErrorCode.BoardNotFound);
    }
    const post = board.posts.find((p) => p.id === postId);
    if (!post) {
      throw new ArtifactError(`post ${postId} not found`, ArtifactErrorCode.PostNotFound);
    }
    post.pinned = false;
  }

  /** Get a specific post from a board. */
  getPost(boardId: BoardId, postId: PostId): Post | undefined {
    const board = this.boards.get(boardId);
    return board?.posts.find((p) => p.id === postId);
  }

  /** Get a thread: root post + all replies (direct and transitive). */
  getThread(boardId: BoardId, rootPostId: PostId): Post[] {
    const board = this.boards.get(boardId);
    if (!board) return [];

    const result: Post[] = [];
    const root = board.posts.find((p) => p.id === rootPostId);
    if (!root) return [];

    result.push(root);

    // BFS to collect all transitive replies
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
      throw new ArtifactError(`board ${boardId} not found`, ArtifactErrorCode.BoardNotFound);
    }
    this.boards.delete(boardId);
  }
}
