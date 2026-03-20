import type { AgentManifest, AgentOrigin } from './manifest.js';
import { TRANSPORT_TYPES, ALL_MODALITIES } from './manifest.js';

export interface ValidationError {
  field: string;
  message: string;
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}

const SEMVER_RE = /^\d+\.\d+\.\d+(?:-[\w.]+)?(?:\+[\w.]+)?$/;
const VALID_ORIGINS = ['rlmx', 'seed', 'external', 'custom'] as const;
const VALID_AUTH_METHODS = ['bearer', 'mtls', 'macaroon', 'none'] as const;

function validateOriginFields(origin: AgentOrigin, errors: ValidationError[]): void {
  switch (origin.type) {
    case 'seed':
      if (!origin.mcpEndpoint) {
        errors.push({ field: 'origin.mcpEndpoint', message: 'seed origin requires mcpEndpoint' });
      }
      break;
    case 'rlmx':
      if (!origin.agentType) {
        errors.push({ field: 'origin.agentType', message: 'rlmx origin requires agentType' });
      }
      break;
    case 'external':
      if (!origin.endpoint) {
        errors.push({ field: 'origin.endpoint', message: 'external origin requires endpoint' });
      }
      break;
    case 'custom':
      if (!origin.handler) {
        errors.push({ field: 'origin.handler', message: 'custom origin requires handler' });
      }
      break;
  }
}

export function validateManifest(manifest: AgentManifest): ValidationResult {
  const errors: ValidationError[] = [];

  // --- Required string fields ---
  if (!manifest.id || typeof manifest.id !== 'string') {
    errors.push({ field: 'id', message: 'id is required and must be a string' });
  }
  if (!manifest.name || typeof manifest.name !== 'string') {
    errors.push({ field: 'name', message: 'name is required and must be a string' });
  }
  if (!manifest.version || !SEMVER_RE.test(manifest.version)) {
    errors.push({ field: 'version', message: 'version must be valid semver (e.g. "1.0.0")' });
  }

  // --- Origin ---
  if (!manifest.origin || typeof manifest.origin.type !== 'string') {
    errors.push({ field: 'origin', message: 'origin is required with a type field' });
  } else {
    if (!(VALID_ORIGINS as readonly string[]).includes(manifest.origin.type)) {
      errors.push({
        field: 'origin.type',
        message: `origin.type must be one of: ${VALID_ORIGINS.join(', ')}`,
      });
    }
    validateOriginFields(manifest.origin, errors);
  }

  // --- Modalities ---
  if (!Array.isArray(manifest.modalities) || manifest.modalities.length === 0) {
    errors.push({ field: 'modalities', message: 'at least one modality is required' });
  } else {
    for (const m of manifest.modalities) {
      if (!(ALL_MODALITIES as readonly string[]).includes(m)) {
        errors.push({ field: 'modalities', message: `invalid modality: ${m}` });
      }
    }
  }

  // --- Transports ---
  if (!Array.isArray(manifest.transports) || manifest.transports.length === 0) {
    errors.push({ field: 'transports', message: 'at least one transport is required' });
  } else {
    for (const t of manifest.transports) {
      if (!(TRANSPORT_TYPES as readonly string[]).includes(t.type)) {
        errors.push({ field: 'transports', message: `invalid transport type: ${t.type}` });
      }
    }
  }

  // --- Resource envelope ---
  if (manifest.resourceEnvelope) {
    const re = manifest.resourceEnvelope;
    if (re.cpuCores <= 0) {
      errors.push({ field: 'resourceEnvelope.cpuCores', message: 'cpuCores must be > 0' });
    }
    if (re.memoryMb <= 0) {
      errors.push({ field: 'resourceEnvelope.memoryMb', message: 'memoryMb must be > 0' });
    }
    if (re.maxRuntimeMs < 0) {
      errors.push({ field: 'resourceEnvelope.maxRuntimeMs', message: 'maxRuntimeMs must be >= 0 (0 means persistent)' });
    }
  } else {
    errors.push({ field: 'resourceEnvelope', message: 'resourceEnvelope is required' });
  }

  // --- Security ---
  if (!manifest.security) {
    errors.push({ field: 'security', message: 'security spec is required' });
  } else {
    if (!(VALID_AUTH_METHODS as readonly string[]).includes(manifest.security.authMethod)) {
      errors.push({
        field: 'security.authMethod',
        message: `authMethod must be one of: ${VALID_AUTH_METHODS.join(', ')}`,
      });
    }
  }

  // --- Deployment ---
  if (!manifest.deployment) {
    errors.push({ field: 'deployment', message: 'deployment spec is required' });
  } else if (!Array.isArray(manifest.deployment.profiles) || manifest.deployment.profiles.length === 0) {
    errors.push({ field: 'deployment.profiles', message: 'at least one deployment profile is required' });
  }

  return { valid: errors.length === 0, errors };
}
