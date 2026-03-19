// ---------------------------------------------------------------------------
// @aix/artifact — Content-addressed artifact DAG for the @aix ecosystem
// ---------------------------------------------------------------------------

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
// SHA-256 helper
// ---------------------------------------------------------------------------

import { createHash } from 'node:crypto';

function sha256Hex(data: Uint8Array): string {
  return createHash('sha256').update(data).digest('hex');
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

