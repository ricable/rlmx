/**
 * Skill validation — ensures skills meet requirements before installation (ADR-035).
 */

import { LIFE_DOMAINS } from '@aix/shared';
import type { Skill } from './types.js';

/** Result of skill validation. */
export interface ValidationResult {
  /** Whether the skill passed all validation checks. */
  valid: boolean;
  /** List of validation errors (empty if valid). */
  errors: string[];
}

/** Semver regex: major.minor.patch with optional pre-release/build. */
const SEMVER_RE = /^\d+\.\d+\.\d+(?:-[\w.]+)?(?:\+[\w.]+)?$/;

/** Valid skill ID: alphanumeric, hyphens, underscores, 1-100 chars. */
const SKILL_ID_RE = /^[a-zA-Z0-9_-]{1,100}$/;

/**
 * Validate a skill's metadata and content.
 * Returns a list of validation errors (empty if valid).
 */
export function validateSkill(skill: Skill): ValidationResult {
  const errors: string[] = [];
  const { metadata, content } = skill;

  // Required fields presence
  if (!metadata.id) {
    errors.push('Missing required field: id');
  } else if (!SKILL_ID_RE.test(metadata.id)) {
    errors.push(
      'Invalid id: must contain only alphanumeric characters, hyphens, and underscores (1-100 chars)',
    );
  }

  if (!metadata.name) {
    errors.push('Missing required field: name');
  }

  if (!metadata.description) {
    errors.push('Missing required field: description');
  }

  if (!metadata.version) {
    errors.push('Missing required field: version');
  } else if (!SEMVER_RE.test(metadata.version)) {
    errors.push(
      `Invalid version format: "${metadata.version}" — must follow semver (x.y.z)`,
    );
  }

  if (!metadata.domain) {
    errors.push('Missing required field: domain');
  } else {
    const validDomains = LIFE_DOMAINS as readonly string[];
    if (!validDomains.includes(metadata.domain)) {
      errors.push(
        `Invalid domain: "${metadata.domain}" — must be one of: ${LIFE_DOMAINS.join(', ')}`,
      );
    }
  }

  // Content must be non-empty
  if (!content || content.trim().length === 0) {
    errors.push('Skill content must be non-empty');
  }

  return { valid: errors.length === 0, errors };
}
