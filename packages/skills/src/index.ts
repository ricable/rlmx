// @aix/skills — Skill system for the @aix ecosystem (ADR-035).

export type {
  SkillSource,
  SkillMetadata,
  Skill,
  SkillFilter,
} from './types.js';

export { parseSkillMd } from './parser.js';

export { SkillRegistry } from './registry.js';

export { validateSkill } from './security.js';
export type { ValidationResult } from './security.js';

export {
  BUNDLED_SKILLS,
  getBundledSkills,
  getBundledSkillByDomain,
} from './bundled.js';
