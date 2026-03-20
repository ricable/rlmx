import type { AgentType } from '@aix/shared';

/**
 * Base error class for all agent-related errors.
 */
export class AgentError extends Error {
  override readonly name: string = 'AgentError';
  constructor(message: string) {
    super(message);
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

/**
 * Thrown when an agent cannot be found by its id.
 */
export class AgentNotFoundError extends AgentError {
  override readonly name = 'AgentNotFoundError';
  constructor(public readonly agentId: string) {
    super(`Agent not found: ${agentId}`);
  }
}

/**
 * Thrown when spawning would exceed the maximum instance count for an agent type.
 */
export class MaxInstancesExceededError extends AgentError {
  override readonly name = 'MaxInstancesExceededError';
  constructor(
    public readonly agentType: AgentType,
    public readonly max: number,
  ) {
    super(`Max instances exceeded for ${agentType}: limit ${max}`);
  }
}

/**
 * Thrown when a parent agent is not allowed to spawn a child of the given type.
 */
export class SpawnNotAllowedError extends AgentError {
  override readonly name = 'SpawnNotAllowedError';
  constructor(
    public readonly parent: AgentType,
    public readonly child: AgentType,
  ) {
    super(`Agent type ${child} cannot be spawned by ${parent}`);
  }
}

/**
 * Thrown when a syscall permission check fails.
 */
export class PermissionDeniedError extends AgentError {
  override readonly name = 'PermissionDeniedError';
  constructor(message: string) {
    super(`Permission denied: ${message}`);
  }
}

/**
 * Thrown when attempting to terminate an already-terminated agent.
 */
export class AlreadyTerminatedError extends AgentError {
  override readonly name = 'AlreadyTerminatedError';
  constructor() {
    super('Agent already terminated');
  }
}

/**
 * Thrown when attempting an invalid lifecycle state transition.
 */
export class InvalidTransitionError extends AgentError {
  override readonly name = 'InvalidTransitionError';
  constructor(
    public readonly from: string,
    public readonly to: string,
  ) {
    super(`Invalid state transition: ${from} -> ${to}`);
  }
}

/**
 * Thrown when the NAPI bridge is unavailable and no fallback exists.
 */
export class NapiBridgeUnavailableError extends AgentError {
  override readonly name = 'NapiBridgeUnavailableError';
  constructor() {
    super(
      'NAPI bridge (@aix/core) is unavailable. ' +
        'Agent spawning requires native bindings or a running MCP server.',
    );
  }
}
