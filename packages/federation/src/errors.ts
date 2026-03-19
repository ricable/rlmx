/**
 * Federation-specific error types.
 *
 * Maps 1:1 to rlmx-federation error::FederationError enum.
 */

/** Discriminated union tag for federation errors. */
export type FederationErrorCode =
  | 'InvalidCycleStatus'
  | 'AggregationThresholdNotMet'
  | 'ContributionRejected'
  | 'NoDomainContributions'
  | 'PackageNotFound'
  | 'CycleNotFound'
  | 'AnonymizationFailed'
  | 'DistributionFailed'
  | 'BootstrapFailed'
  | 'PrivacyViolation';

/**
 * Base error class for the federation bounded context.
 */
export class FederationError extends Error {
  readonly code: FederationErrorCode;
  readonly details?: Record<string, unknown>;

  constructor(
    code: FederationErrorCode,
    message: string,
    details?: Record<string, unknown>,
  ) {
    super(message);
    this.name = 'FederationError';
    this.code = code;
    this.details = details;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

// ---------------------------------------------------------------------------
// Factory helpers
// ---------------------------------------------------------------------------

export function invalidCycleStatus(cycleId: string): FederationError {
  return new FederationError(
    'InvalidCycleStatus',
    `Cycle ${cycleId} is not in the expected status for this operation`,
    { cycleId },
  );
}

export function aggregationThresholdNotMet(
  required: number,
  actual: number,
): FederationError {
  return new FederationError(
    'AggregationThresholdNotMet',
    `Aggregation threshold not met: need ${required} contributors, have ${actual}`,
    { required, actual },
  );
}

export function contributionRejected(reason: string): FederationError {
  return new FederationError(
    'ContributionRejected',
    `Contribution rejected: ${reason}`,
    { reason },
  );
}

export function noDomainContributions(domain: string): FederationError {
  return new FederationError(
    'NoDomainContributions',
    `No contributions for domain ${domain} in this cycle`,
    { domain },
  );
}

export function packageNotFound(packageId: string): FederationError {
  return new FederationError(
    'PackageNotFound',
    `Package not found: ${packageId}`,
    { packageId },
  );
}

export function cycleNotFound(cycleId: string): FederationError {
  return new FederationError(
    'CycleNotFound',
    `Cycle not found: ${cycleId}`,
    { cycleId },
  );
}

export function anonymizationFailed(reason: string): FederationError {
  return new FederationError(
    'AnonymizationFailed',
    `Anonymization failed: ${reason}`,
    { reason },
  );
}

export function distributionFailed(reason: string): FederationError {
  return new FederationError(
    'DistributionFailed',
    `Distribution failed: ${reason}`,
    { reason },
  );
}

export function bootstrapFailed(reason: string): FederationError {
  return new FederationError(
    'BootstrapFailed',
    `Bootstrap failed: ${reason}`,
    { reason },
  );
}

export function privacyViolation(reason: string): FederationError {
  return new FederationError(
    'PrivacyViolation',
    `Privacy invariant violated: ${reason}`,
    { reason },
  );
}
