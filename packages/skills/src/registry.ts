/**
 * Skill registry — manages installed skills (ADR-035).
 */

import type { Skill, SkillFilter } from './types.js';
import { validateSkill } from './security.js';

/**
 * In-memory registry for skills.
 * Supports install, uninstall, lookup, listing, and search.
 */
export class SkillRegistry {
  private readonly skills = new Map<string, Skill>();

  /**
   * Install a skill into the registry.
   * Validates the skill before installation.
   *
   * @throws Error if validation fails or skill ID already exists.
   */
  install(skill: Skill): void {
    const result = validateSkill(skill);
    if (!result.valid) {
      throw new Error(
        `Skill validation failed: ${result.errors.join('; ')}`,
      );
    }

    if (this.skills.has(skill.metadata.id)) {
      throw new Error(
        `Skill already installed: ${skill.metadata.id}`,
      );
    }

    const installed: Skill = {
      ...skill,
      installedAt: skill.installedAt ?? new Date().toISOString(),
    };

    this.skills.set(installed.metadata.id, installed);
  }

  /**
   * Uninstall a skill by ID.
   * Bundled skills cannot be uninstalled.
   *
   * @throws Error if skill not found or is bundled.
   */
  uninstall(id: string): void {
    const skill = this.skills.get(id);
    if (!skill) {
      throw new Error(`Skill not found: ${id}`);
    }
    if (skill.source === 'bundled') {
      throw new Error(`Cannot uninstall bundled skill: ${id}`);
    }
    this.skills.delete(id);
  }

  /** Retrieve a skill by ID, or `undefined` if not found. */
  get(id: string): Skill | undefined {
    return this.skills.get(id);
  }

  /** List all skills, optionally filtered. */
  list(filter?: SkillFilter): Skill[] {
    let results = Array.from(this.skills.values());

    if (filter?.domain) {
      results = results.filter((s) => s.metadata.domain === filter.domain);
    }
    if (filter?.source) {
      results = results.filter((s) => s.source === filter.source);
    }
    if (filter?.category) {
      results = results.filter((s) => s.metadata.category === filter.category);
    }

    return results;
  }

  /**
   * Search skills by query string.
   * Performs case-insensitive substring match on name, description, and tags.
   */
  search(query: string): Skill[] {
    const q = query.toLowerCase();
    return Array.from(this.skills.values()).filter((skill) => {
      const { name, description, tags } = skill.metadata;
      return (
        name.toLowerCase().includes(q) ||
        description.toLowerCase().includes(q) ||
        tags.some((tag) => tag.toLowerCase().includes(q))
      );
    });
  }

  /** Number of skills in the registry. */
  get size(): number {
    return this.skills.size;
  }
}
