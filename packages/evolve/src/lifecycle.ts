import {
  type EvolvedFunction,
  type FunctionStatus,
  type AutoScoreMode,
  canTransition,
  EvolveError,
  generateId,
} from './types.js';

/**
 * Manages the lifecycle of evolved functions.
 */
export class LifecycleManager {
  private functions = new Map<string, EvolvedFunction>();

  /**
   * Create a new function in Draft status.
   * Returns the assigned function ID.
   */
  create(
    name: string,
    code: string,
    goal: string,
    parentId: string | null = null,
    scoreMode: AutoScoreMode = 'auto',
  ): string {
    const id = generateId();

    const existingVersions = this.byName(name);
    const maxVersion = existingVersions.reduce(
      (max, f) => Math.max(max, f.version),
      0,
    );

    const func: EvolvedFunction = {
      id,
      name,
      version: maxVersion + 1,
      code,
      goal,
      status: 'draft',
      scoreMode,
      createdAt: new Date().toISOString(),
      parentId,
      metadata: {},
    };

    this.functions.set(id, func);
    return id;
  }

  /**
   * Transition a function to a new status.
   * Throws EvolveError if the transition is invalid.
   */
  transition(id: string, newStatus: FunctionStatus): void {
    const func = this.functions.get(id);
    if (!func) {
      throw new EvolveError('function not found', 'FUNCTION_NOT_FOUND');
    }

    if (!canTransition(func.status, newStatus)) {
      throw new EvolveError(
        `invalid transition from ${func.status} to ${newStatus}`,
        'INVALID_TRANSITION',
      );
    }

    func.status = newStatus;
  }

  /** Get a function by ID. */
  get(id: string): EvolvedFunction | undefined {
    return this.functions.get(id);
  }

  /** List all functions with a given status. */
  listByStatus(status: FunctionStatus): EvolvedFunction[] {
    return Array.from(this.functions.values()).filter(
      (f) => f.status === status,
    );
  }

  /** List all functions. */
  listAll(): EvolvedFunction[] {
    return Array.from(this.functions.values());
  }

  /** Get all versions of a function by name. */
  byName(name: string): EvolvedFunction[] {
    return Array.from(this.functions.values()).filter((f) => f.name === name);
  }
}
