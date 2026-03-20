import type { AgentStatus } from './types.js';
import { InvalidTransitionError } from './errors.js';

// ---------------------------------------------------------------------------
// Valid transitions — encoded as a static adjacency list
// ---------------------------------------------------------------------------

const VALID_TRANSITIONS: ReadonlyMap<AgentStatus, readonly AgentStatus[]> = new Map([
  ['Pending', ['Starting']],
  ['Starting', ['Running', 'Failed']],
  ['Running', ['Paused', 'Stopping', 'Failed']],
  ['Paused', ['Running', 'Stopping']],
  ['Stopping', ['Terminated']],
  // Terminal states have no outgoing transitions
  ['Terminated', []],
  ['Failed', []],
]);

// ---------------------------------------------------------------------------
// AgentLifecycle — state machine
// ---------------------------------------------------------------------------

/**
 * Agent lifecycle state machine (ADR-005).
 *
 * Valid transitions:
 *   Pending -> Starting -> Running -> Paused -> Running (resume)
 *                                  -> Stopping -> Terminated
 *                       -> Failed (from Starting or Running)
 *
 * Invalid transitions throw InvalidTransitionError.
 */
export class AgentLifecycle {
  private _status: AgentStatus;

  constructor(initial: AgentStatus = 'Pending') {
    this._status = initial;
  }

  /** Current lifecycle status. */
  get status(): AgentStatus {
    return this._status;
  }

  /**
   * Returns true if transitioning from `current` to `next` is valid.
   */
  static isValidTransition(current: AgentStatus, next: AgentStatus): boolean {
    const allowed = VALID_TRANSITIONS.get(current);
    return allowed !== undefined && allowed.includes(next);
  }

  /**
   * Attempts a transition, returning the new state.
   * Throws InvalidTransitionError if the transition is not allowed.
   */
  static transition(current: AgentStatus, next: AgentStatus): AgentStatus {
    if (!AgentLifecycle.isValidTransition(current, next)) {
      throw new InvalidTransitionError(current, next);
    }
    return next;
  }

  /**
   * Returns all valid next states from the given state.
   */
  static validNextStates(current: AgentStatus): readonly AgentStatus[] {
    return VALID_TRANSITIONS.get(current) ?? [];
  }

  /**
   * Returns true if the state is terminal (no further transitions possible).
   */
  static isTerminal(status: AgentStatus): boolean {
    return status === 'Terminated' || status === 'Failed';
  }

  // -----------------------------------------------------------------------
  // Instance methods — mutate internal state
  // -----------------------------------------------------------------------

  /**
   * Transition to a new state, mutating this instance.
   * Throws InvalidTransitionError if the transition is not allowed.
   */
  transitionTo(next: AgentStatus): void {
    this._status = AgentLifecycle.transition(this._status, next);
  }

  /** Whether the agent is in a terminal state. */
  get isTerminal(): boolean {
    return AgentLifecycle.isTerminal(this._status);
  }

  /** All valid states this agent can move to from its current state. */
  get validNextStates(): readonly AgentStatus[] {
    return AgentLifecycle.validNextStates(this._status);
  }
}
