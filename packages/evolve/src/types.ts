import { generateId } from '@aix/shared';

/** Lifecycle status of an evolved function. */
export type FunctionStatus =
  | 'draft'
  | 'staging'
  | 'production'
  | 'deprecated'
  | 'killed';

/** Auto-scoring mode for a function. */
export type AutoScoreMode = 'auto' | 'sampled' | 'manual' | 'off';

/** A versioned, evolvable function managed by the evolution engine. */
export interface EvolvedFunction {
  id: string;
  name: string;
  version: number;
  code: string;
  goal: string;
  status: FunctionStatus;
  scoreMode: AutoScoreMode;
  createdAt: string;
  parentId: string | null;
  metadata: Record<string, string>;
}

/** Result of scoring a single function execution. */
export interface ScoreResult {
  overall: number;
  correctness: number;
  safety: number;
  latencyScore: number;
  costScore: number;
}

/** Decision produced by the feedback engine. */
export type FeedbackDecision =
  | { type: 'keep' }
  | { type: 'improve'; reason: string }
  | { type: 'kill'; reason: string };

/** Valid state machine transitions for FunctionStatus. */
const VALID_TRANSITIONS: ReadonlyMap<FunctionStatus, readonly FunctionStatus[]> =
  new Map<FunctionStatus, readonly FunctionStatus[]>([
    ['draft', ['staging', 'killed']],
    ['staging', ['production', 'killed']],
    ['production', ['deprecated', 'killed']],
    ['deprecated', ['killed']],
    ['killed', []],
  ]);

/**
 * Check whether a transition from one status to another is valid.
 */
export function canTransition(
  from: FunctionStatus,
  to: FunctionStatus,
): boolean {
  const allowed = VALID_TRANSITIONS.get(from);
  return allowed !== undefined && allowed.includes(to);
}

/**
 * Error class for evolution operations.
 */
export class EvolveError extends Error {
  constructor(
    message: string,
    public readonly code: string,
  ) {
    super(message);
    this.name = 'EvolveError';
    Object.setPrototypeOf(this, EvolveError.prototype);
  }
}

export { generateId };
