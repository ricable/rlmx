/**
 * Skill system types (ADR-035).
 */

import type { LifeDomain } from '@aix/shared';

/** Where a skill originated from. */
export type SkillSource = 'bundled' | 'installed' | 'marketplace';

/** Metadata describing a skill's identity and requirements. */
export interface SkillMetadata {
  /** Unique identifier (alphanumeric, hyphens, underscores). */
  id: string;
  /** Human-readable name. */
  name: string;
  /** Description of what the skill does. */
  description: string;
  /** Semver version string. */
  version: string;
  /** Broad category (e.g., "advisory", "automation", "analysis"). */
  category: string;
  /** Searchable tags. */
  tags: string[];
  /** The life domain this skill serves. */
  domain: LifeDomain;
  /** Capabilities required by this skill. */
  capabilities: {
    /** Kernel tools this skill needs access to. */
    tools: string[];
    /** Memory scopes this skill operates on. */
    memoryScopes: string[];
  };
}

/** A complete skill definition with metadata, source, and content. */
export interface Skill {
  /** Skill metadata from frontmatter. */
  metadata: SkillMetadata;
  /** Where this skill came from. */
  source: SkillSource;
  /** The skill body (system prompt + instructions). */
  content: string;
  /** ISO timestamp of when the skill was installed. */
  installedAt?: string;
}

/** Filter criteria for listing skills. */
export interface SkillFilter {
  /** Filter by life domain. */
  domain?: LifeDomain;
  /** Filter by source. */
  source?: SkillSource;
  /** Filter by category. */
  category?: string;
}
