// ---------------------------------------------------------------------------
// @aix/swarm — BrowserComputePool with priority queue
// Port of rlmx-swarm/src/browser_pool.rs.
// ---------------------------------------------------------------------------

import { randomUUID } from 'node:crypto';

// -- Types ------------------------------------------------------------------

/** Status of a browser compute worker. */
export enum WorkerStatus {
  Idle = 'Idle',
  Computing = 'Computing',
  Disconnected = 'Disconnected',
}

/** Capabilities reported by a browser worker on connection. */
export interface BrowserCapabilities {
  wasm: boolean;
  simd: boolean;
  webgpu: boolean;
  sharedArrayBuffer: boolean;
  maxWorkers: number;
}

/** Default browser capabilities. */
export function defaultCapabilities(): BrowserCapabilities {
  return {
    wasm: true,
    simd: false,
    webgpu: false,
    sharedArrayBuffer: false,
    maxWorkers: 1,
  };
}

/** A browser-based compute worker. */
export interface BrowserWorker {
  id: string;
  connectedAt: string;
  tasksCompleted: number;
  status: WorkerStatus;
  capabilities?: BrowserCapabilities;
}

/** Type of compute task to offload to browser workers. */
export type ComputeTaskType =
  | { type: 'Embedding'; text: string; dim: number }
  | { type: 'MatMul'; aShape: [number, number]; bShape: [number, number] }
  | { type: 'CosineSimilarity'; query: number[]; candidatesCount: number }
  | { type: 'TrmForward'; stream: string; inputDim: number };

/** A compute task to be executed by a browser worker. */
export interface ComputeTask {
  id: string;
  taskType: ComputeTaskType;
  data: Uint8Array;
  createdAt: string;
  /** Priority level: 0 = highest (embedding), 1 = TRM, 2 = experiment, 3 = background. */
  priority: number;
}

/** An entry tracking a task currently being computed by a worker. */
export interface InFlightEntry {
  task: ComputeTask;
  workerId: string;
  assignedAt: string;
}

// -- Priority Queue ---------------------------------------------------------

/**
 * Simple priority queue using sorted insertion.
 * Lower priority number = higher urgency (min-heap semantics).
 * Ties broken by earlier createdAt.
 */
class PriorityQueue {
  private items: ComputeTask[] = [];

  /** Insert using binary search — O(log n) search + O(n) splice, avoids full sort. */
  push(task: ComputeTask): void {
    let lo = 0;
    let hi = this.items.length;
    while (lo < hi) {
      const mid = (lo + hi) >>> 1;
      const m = this.items[mid];
      const cmp = task.priority - m.priority ||
        (task.createdAt < m.createdAt ? -1 : task.createdAt > m.createdAt ? 1 : 0);
      if (cmp < 0) hi = mid;
      else lo = mid + 1;
    }
    this.items.splice(lo, 0, task);
  }

  pop(): ComputeTask | undefined {
    return this.items.shift();
  }

  get length(): number {
    return this.items.length;
  }
}

// -- BrowserComputePool -----------------------------------------------------

const DEFAULT_TIMEOUT_MS = 30_000;

/**
 * Pool of browser-based compute workers with priority queue and fault tolerance.
 * Port of Rust BrowserComputePool.
 */
export class BrowserComputePool {
  readonly workers: Map<string, BrowserWorker> = new Map();
  private pendingTasks = new PriorityQueue();
  private completedResults: Map<string, Uint8Array> = new Map();
  private inFlight: Map<string, InFlightEntry> = new Map();
  readonly timeoutMs: number;

  constructor(timeoutMs?: number) {
    this.timeoutMs = timeoutMs ?? DEFAULT_TIMEOUT_MS;
  }

  /** Register a new browser worker with optional capabilities. */
  registerWorker(id: string, capabilities?: BrowserCapabilities): void {
    this.workers.set(id, {
      id,
      connectedAt: new Date().toISOString(),
      tasksCompleted: 0,
      status: WorkerStatus.Idle,
      capabilities,
    });
  }

  /** Unregister a worker. All in-flight tasks for this worker are re-queued. */
  unregisterWorker(id: string): void {
    const requeue: string[] = [];
    for (const [taskId, entry] of this.inFlight) {
      if (entry.workerId === id) requeue.push(taskId);
    }
    for (const taskId of requeue) {
      const entry = this.inFlight.get(taskId)!;
      this.inFlight.delete(taskId);
      this.pendingTasks.push(entry.task);
    }
    this.workers.delete(id);
  }

  /** Submit a compute task. Returns the task ID. */
  submitTask(task: ComputeTask): string {
    const id = task.id;
    this.pendingTasks.push(task);
    return id;
  }

  /** Poll for the next available task (called by workers). Respects priority ordering. */
  pollTask(workerId: string): ComputeTask | undefined {
    const worker = this.workers.get(workerId);
    if (!worker || worker.status === WorkerStatus.Disconnected) return undefined;

    const task = this.pendingTasks.pop();
    if (!task) return undefined;

    worker.status = WorkerStatus.Computing;
    this.inFlight.set(task.id, {
      task,
      workerId,
      assignedAt: new Date().toISOString(),
    });
    return task;
  }

  /**
   * Mark a task as complete with its result.
   * Returns true if accepted, false if duplicate/expired.
   */
  completeTask(taskId: string, workerId: string, result: Uint8Array): boolean {
    const entry = this.inFlight.get(taskId);
    if (!entry) return false;

    this.inFlight.delete(taskId);
    this.completedResults.set(taskId, result);

    const worker = this.workers.get(entry.workerId);
    if (worker) {
      worker.tasksCompleted += 1;
      worker.status = WorkerStatus.Idle;
    }
    return true;
  }

  /**
   * Reap expired in-flight tasks back to the pending queue.
   * Returns the number of tasks re-queued.
   */
  reapExpired(): number {
    const now = Date.now();
    const expiredIds: string[] = [];

    for (const [taskId, entry] of this.inFlight) {
      const assignedMs = new Date(entry.assignedAt).getTime();
      if (now - assignedMs > this.timeoutMs) {
        expiredIds.push(taskId);
      }
    }

    for (const taskId of expiredIds) {
      const entry = this.inFlight.get(taskId)!;
      this.inFlight.delete(taskId);
      const worker = this.workers.get(entry.workerId);
      if (worker) worker.status = WorkerStatus.Idle;
      this.pendingTasks.push(entry.task);
    }

    return expiredIds.length;
  }

  /** Retrieve and remove a completed result by task ID. */
  getResult(taskId: string): Uint8Array | undefined {
    const result = this.completedResults.get(taskId);
    if (result !== undefined) this.completedResults.delete(taskId);
    return result;
  }

  /** Number of registered workers. */
  get workerCount(): number {
    return this.workers.size;
  }

  /** Number of pending tasks in the queue. */
  get pendingCount(): number {
    return this.pendingTasks.length;
  }

  /** Number of tasks currently in flight. */
  get inFlightCount(): number {
    return this.inFlight.size;
  }
}

/** Create a ComputeTask with defaults. */
export function createComputeTask(
  taskType: ComputeTaskType,
  priority: number,
  data: Uint8Array = new Uint8Array(),
): ComputeTask {
  return {
    id: randomUUID(),
    taskType,
    data,
    createdAt: new Date().toISOString(),
    priority,
  };
}
