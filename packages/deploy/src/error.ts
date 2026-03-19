import { AixError } from '@aix/shared';

export enum DeployErrorCode {
  ManifestValidation = -36001,
  ManifestNotFound = -36002,
  DuplicateManifest = -36003,
  TransportUnavailable = -36004,
  DiscoveryFailed = -36005,
  SeedBridgeError = -36006,
  ProfileNotFound = -36007,
  TemplateNotFound = -36008,
  DimensionMismatch = -36009,
  GeneratorError = -36010,
}

/**
 * Deploy-specific error. Extends AixError for ecosystem consistency.
 * Uses -36xxx code range.
 */
export class DeployError extends AixError {
  constructor(code: DeployErrorCode, message: string, details?: unknown) {
    super(code as unknown as number, message, details);
    this.name = 'DeployError';
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

// ---------------------------------------------------------------------------
// Factory helpers
// ---------------------------------------------------------------------------

export function manifestValidation(message: string, details?: unknown): DeployError {
  return new DeployError(DeployErrorCode.ManifestValidation, message, details);
}

export function manifestNotFound(id: string): DeployError {
  return new DeployError(DeployErrorCode.ManifestNotFound, `Manifest not found: ${id}`, { id });
}

export function duplicateManifest(id: string): DeployError {
  return new DeployError(DeployErrorCode.DuplicateManifest, `Manifest already registered: ${id}`, { id });
}

export function transportUnavailable(protocol: string): DeployError {
  return new DeployError(DeployErrorCode.TransportUnavailable, `Transport unavailable: ${protocol}`, { protocol });
}

export function discoveryFailed(message: string): DeployError {
  return new DeployError(DeployErrorCode.DiscoveryFailed, message);
}

export function seedBridgeError(message: string, details?: unknown): DeployError {
  return new DeployError(DeployErrorCode.SeedBridgeError, message, details);
}

export function profileNotFound(name: string): DeployError {
  return new DeployError(DeployErrorCode.ProfileNotFound, `Deployment profile not found: ${name}`, { name });
}

export function templateNotFound(name: string): DeployError {
  return new DeployError(DeployErrorCode.TemplateNotFound, `Agent template not found: ${name}`, { name });
}

export function dimensionMismatch(expected: number, got: number): DeployError {
  return new DeployError(
    DeployErrorCode.DimensionMismatch,
    `Vector dimension mismatch: expected ${expected}, got ${got}`,
    { expected, got },
  );
}

export function generatorError(message: string): DeployError {
  return new DeployError(DeployErrorCode.GeneratorError, message);
}
