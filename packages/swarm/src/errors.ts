// ---------------------------------------------------------------------------
// @aix/swarm — Error types for swarm operations
// Maps 1:1 to rlmx-swarm SwarmError and SandboxError.
// ---------------------------------------------------------------------------

/**
 * Error codes for swarm operations.
 */
export enum SwarmErrorCode {
  NodeNotFound = 'NODE_NOT_FOUND',
  ZoneNotFound = 'ZONE_NOT_FOUND',
  ClusterFull = 'CLUSTER_FULL',
  ConsensusFailed = 'CONSENSUS_FAILED',
  TransportError = 'TRANSPORT_ERROR',
  HealthCheckFailed = 'HEALTH_CHECK_FAILED',
  CloudError = 'CLOUD_ERROR',
  NotConfigured = 'NOT_CONFIGURED',
  InvalidOperation = 'INVALID_OPERATION',
  SandboxNotFound = 'SANDBOX_NOT_FOUND',
  ProfileNotFound = 'PROFILE_NOT_FOUND',
  InvalidTransition = 'INVALID_TRANSITION',
  DuplicateProfile = 'DUPLICATE_PROFILE',
  FleetValidation = 'FLEET_VALIDATION',
}

/**
 * Structured error for all swarm operations.
 */
export class SwarmError extends Error {
  constructor(
    message: string,
    public readonly code: SwarmErrorCode,
    public readonly details?: Record<string, unknown>,
  ) {
    super(message);
    this.name = 'SwarmError';
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

/** Convenience constructors matching Rust error variants. */
export function nodeNotFound(nodeId: string): SwarmError {
  return new SwarmError(`node not found: ${nodeId}`, SwarmErrorCode.NodeNotFound, { nodeId });
}

export function zoneNotFound(zoneId: string): SwarmError {
  return new SwarmError(`zone not found: ${zoneId}`, SwarmErrorCode.ZoneNotFound, { zoneId });
}

export function clusterFull(max: number): SwarmError {
  return new SwarmError(`cluster full: max ${max} nodes`, SwarmErrorCode.ClusterFull, { max });
}

export function consensusFailed(reason: string): SwarmError {
  return new SwarmError(`consensus failed: ${reason}`, SwarmErrorCode.ConsensusFailed);
}

export function sandboxNotFound(sandboxId: string): SwarmError {
  return new SwarmError(
    `sandbox not found: ${sandboxId}`,
    SwarmErrorCode.SandboxNotFound,
    { sandboxId },
  );
}

export function profileNotFound(profileName: string): SwarmError {
  return new SwarmError(
    `profile not found: ${profileName}`,
    SwarmErrorCode.ProfileNotFound,
    { profileName },
  );
}

export function invalidTransition(from: string, to: string): SwarmError {
  return new SwarmError(
    `invalid state transition: ${from} -> ${to}`,
    SwarmErrorCode.InvalidTransition,
    { from, to },
  );
}

export function duplicateProfile(name: string): SwarmError {
  return new SwarmError(
    `duplicate profile: ${name}`,
    SwarmErrorCode.DuplicateProfile,
    { name },
  );
}

export function fleetValidation(reason: string): SwarmError {
  return new SwarmError(reason, SwarmErrorCode.FleetValidation);
}
