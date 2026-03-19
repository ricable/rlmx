/**
 * In-memory task store with FIFO eviction (ADR-034).
 */

import { generateId } from '@aix/shared';
import type { A2AMessage, A2ATask, TaskState } from './types.js';
import { canTransition } from './types.js';

/** Maximum number of tasks before FIFO eviction kicks in. */
const MAX_TASKS = 1_000;

/**
 * In-memory store for A2A tasks.
 * Enforces FIFO eviction at {@link MAX_TASKS} entries.
 */
export class TaskStore {
  private readonly tasks = new Map<string, A2ATask>();
  private readonly maxTasks: number;

  constructor(maxTasks: number = MAX_TASKS) {
    this.maxTasks = maxTasks;
  }

  /** Create a new task from an initial user message. */
  create(message: A2AMessage): A2ATask {
    this.evictIfNeeded();

    const task: A2ATask = {
      id: generateId(),
      state: 'submitted',
      messages: [message],
    };

    this.tasks.set(task.id, task);
    return task;
  }

  /** Retrieve a task by ID, or `undefined` if not found. */
  get(id: string): A2ATask | undefined {
    return this.tasks.get(id);
  }

  /**
   * Update a task's state and optionally append a message.
   * Validates state transitions. Throws on invalid transitions.
   */
  update(id: string, state: TaskState, message?: A2AMessage): A2ATask {
    const task = this.tasks.get(id);
    if (!task) {
      throw new Error(`Task not found: ${id}`);
    }

    if (!canTransition(task.state, state)) {
      throw new Error(
        `Invalid state transition: ${task.state} -> ${state}`,
      );
    }

    task.state = state;
    if (message) {
      task.messages.push(message);
    }

    return task;
  }

  /** Cancel a task. Throws if the task cannot be cancelled. */
  cancel(id: string): A2ATask {
    return this.update(id, 'cancelled');
  }

  /** List tasks, optionally filtered by state. */
  list(state?: TaskState): A2ATask[] {
    const all = Array.from(this.tasks.values());
    if (state === undefined) {
      return all;
    }
    return all.filter((t) => t.state === state);
  }

  /** Number of tasks currently stored. */
  get size(): number {
    return this.tasks.size;
  }

  /** Evict the oldest task if at capacity. Uses Map insertion order for O(1). */
  private evictIfNeeded(): void {
    while (this.tasks.size >= this.maxTasks) {
      // Map.keys() iterates in insertion order; first key is oldest
      const oldest = this.tasks.keys().next().value;
      if (oldest === undefined) break;
      this.tasks.delete(oldest);
    }
  }
}
