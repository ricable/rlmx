/**
 * Standardized error codes for the @aix ecosystem.
 * Combines kernel errors, agent errors, and JSON-RPC standard codes.
 */
export enum AixErrorCode {
  // JSON-RPC standard errors
  ParseError = -32700,
  InvalidRequest = -32600,
  MethodNotFound = -32601,
  InvalidParams = -32602,
  InternalError = -32603,

  // Kernel errors (-33xxx)
  CapabilityDenied = -33001,
  ProcessNotFound = -33002,
  SegmentNotFound = -33003,
  GraphError = -33004,
  ProofError = -33005,
  SchedulerError = -33006,
  Timeout = -33007,
  ChannelError = -33008,

  // Agent errors (-34xxx)
  AgentNotFound = -34001,
  MaxInstancesExceeded = -34002,
  SpawnNotAllowed = -34003,
  PermissionDenied = -34004,
  AgentAlreadyTerminated = -34005,

  // Billing errors (-35xxx)
  AgentLimitReached = -35001,
  CloudTokensExhausted = -35002,
  FeatureUnavailable = -35003,
}

/**
 * Base error class for the @aix ecosystem.
 * Carries a numeric error code suitable for JSON-RPC error responses.
 */
export class AixError extends Error {
  readonly code: AixErrorCode;
  readonly details?: unknown;

  constructor(code: AixErrorCode, message: string, details?: unknown) {
    super(message);
    this.name = 'AixError';
    this.code = code;
    this.details = details;
    // Maintain proper prototype chain for instanceof checks
    Object.setPrototypeOf(this, new.target.prototype);
  }

  /** Serialize to a plain object suitable for JSON-RPC error field. */
  toJSON(): { code: number; message: string; data?: unknown } {
    return {
      code: this.code,
      message: this.message,
      ...(this.details !== undefined ? { data: this.details } : {}),
    };
  }
}

// ---------------------------------------------------------------------------
// Factory helpers for common errors
// ---------------------------------------------------------------------------

export function capabilityDenied(permission: string): AixError {
  return new AixError(
    AixErrorCode.CapabilityDenied,
    `Capability denied: ${permission}`,
    { permission },
  );
}

export function processNotFound(processId: string): AixError {
  return new AixError(
    AixErrorCode.ProcessNotFound,
    `Process not found: ${processId}`,
    { processId },
  );
}

export function methodNotFound(method: string): AixError {
  return new AixError(
    AixErrorCode.MethodNotFound,
    `Method not found: ${method}`,
    { method },
  );
}

export function invalidParams(message: string, details?: unknown): AixError {
  return new AixError(AixErrorCode.InvalidParams, message, details);
}

export function internalError(message: string, details?: unknown): AixError {
  return new AixError(AixErrorCode.InternalError, message, details);
}

export function agentLimitReached(current: number, limit: number): AixError {
  return new AixError(
    AixErrorCode.AgentLimitReached,
    `Agent limit reached: ${current}/${limit}`,
    { current, limit },
  );
}

export function featureUnavailable(feature: string): AixError {
  return new AixError(
    AixErrorCode.FeatureUnavailable,
    `Feature unavailable: ${feature}`,
    { feature },
  );
}

export function timeout(durationMs: number): AixError {
  return new AixError(
    AixErrorCode.Timeout,
    `Timeout after ${durationMs}ms`,
    { durationMs },
  );
}
