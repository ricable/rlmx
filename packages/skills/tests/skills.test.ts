import { describe, it, expect, beforeEach } from 'vitest';
import { LifeDomain, LIFE_DOMAINS } from '@aix/shared';
import {
  parseSkillMd,
  SkillRegistry,
  validateSkill,
  BUNDLED_SKILLS,
  getBundledSkills,
  getBundledSkillByDomain,
} from '../src/index.js';
import type { Skill } from '../src/index.js';

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

function makeSkill(overrides?: Partial<Skill>): Skill {
  return {
    metadata: {
      id: 'test-skill',
      name: 'Test Skill',
      description: 'A test skill for unit tests',
      version: '1.0.0',
      category: 'testing',
      tags: ['test'],
      domain: LifeDomain.Finance,
      capabilities: { tools: ['vec-search'], memoryScopes: ['test-scope'] },
    },
    source: 'installed',
    content: 'You are a test skill agent.',
    ...overrides,
  };
}

const SAMPLE_MD = `---
id: sample-skill
name: Sample Skill
description: A sample skill for testing
version: 2.0.0
category: advisory
tags: [finance, budget]
domain: Finance
capabilities:
  tools: [vec-search, graph-query]
  memoryScopes: [financial-history]
---
You are a sample skill agent. Help users with financial questions.`;

// ---------------------------------------------------------------------------
// parseSkillMd
// ---------------------------------------------------------------------------

describe('parseSkillMd', () => {
  it('parses valid skill markdown', () => {
    const skill = parseSkillMd(SAMPLE_MD);
    expect(skill.metadata.id).toBe('sample-skill');
    expect(skill.metadata.name).toBe('Sample Skill');
    expect(skill.metadata.version).toBe('2.0.0');
    expect(skill.metadata.category).toBe('advisory');
    expect(skill.metadata.domain).toBe('Finance');
    expect(skill.metadata.tags).toEqual(['finance', 'budget']);
    expect(skill.content).toContain('sample skill agent');
  });

  it('sets source to installed', () => {
    const skill = parseSkillMd(SAMPLE_MD);
    expect(skill.source).toBe('installed');
  });

  it('parses capabilities', () => {
    const skill = parseSkillMd(SAMPLE_MD);
    expect(skill.metadata.capabilities.tools).toEqual([
      'vec-search',
      'graph-query',
    ]);
    expect(skill.metadata.capabilities.memoryScopes).toEqual([
      'financial-history',
    ]);
  });

  it('throws on missing frontmatter', () => {
    expect(() => parseSkillMd('No frontmatter here')).toThrow(
      'must start with YAML frontmatter',
    );
  });

  it('throws on unterminated frontmatter', () => {
    expect(() => parseSkillMd('---\nid: test\n')).toThrow(
      'Unterminated YAML frontmatter',
    );
  });

  it('throws on missing required field', () => {
    const md = `---
name: Missing ID
description: test
version: 1.0.0
domain: Finance
---
Content here.`;
    expect(() => parseSkillMd(md)).toThrow('id');
  });

  it('handles quoted string values', () => {
    const md = `---
id: quoted-test
name: "Quoted Name"
description: 'Single quoted'
version: 1.0.0
domain: Finance
category: test
tags: []
---
Content.`;
    const skill = parseSkillMd(md);
    expect(skill.metadata.name).toBe('Quoted Name');
    expect(skill.metadata.description).toBe('Single quoted');
  });

  it('handles empty tags array', () => {
    const md = `---
id: no-tags
name: No Tags
description: Skill without tags
version: 1.0.0
domain: Finance
category: test
tags: []
---
Content.`;
    const skill = parseSkillMd(md);
    expect(skill.metadata.tags).toEqual([]);
  });
});

// ---------------------------------------------------------------------------
// validateSkill
// ---------------------------------------------------------------------------

describe('validateSkill', () => {
  it('validates a correct skill', () => {
    const result = validateSkill(makeSkill());
    expect(result.valid).toBe(true);
    expect(result.errors).toHaveLength(0);
  });

  it('rejects missing id', () => {
    const skill = makeSkill();
    skill.metadata.id = '';
    const result = validateSkill(skill);
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.includes('id'))).toBe(true);
  });

  it('rejects invalid id characters', () => {
    const skill = makeSkill();
    skill.metadata.id = 'invalid skill id!';
    const result = validateSkill(skill);
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.includes('Invalid id'))).toBe(true);
  });

  it('rejects missing name', () => {
    const skill = makeSkill();
    skill.metadata.name = '';
    const result = validateSkill(skill);
    expect(result.valid).toBe(false);
  });

  it('rejects missing description', () => {
    const skill = makeSkill();
    skill.metadata.description = '';
    const result = validateSkill(skill);
    expect(result.valid).toBe(false);
  });

  it('rejects missing version', () => {
    const skill = makeSkill();
    skill.metadata.version = '';
    const result = validateSkill(skill);
    expect(result.valid).toBe(false);
  });

  it('rejects invalid semver', () => {
    const skill = makeSkill();
    skill.metadata.version = 'not-semver';
    const result = validateSkill(skill);
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.includes('semver'))).toBe(true);
  });

  it('accepts semver with pre-release', () => {
    const skill = makeSkill();
    skill.metadata.version = '1.0.0-alpha.1';
    const result = validateSkill(skill);
    expect(result.valid).toBe(true);
  });

  it('rejects invalid domain', () => {
    const skill = makeSkill();
    skill.metadata.domain = 'InvalidDomain' as LifeDomain;
    const result = validateSkill(skill);
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.includes('domain'))).toBe(true);
  });

  it('rejects empty content', () => {
    const skill = makeSkill({ content: '' });
    const result = validateSkill(skill);
    expect(result.valid).toBe(false);
    expect(result.errors.some((e) => e.includes('content'))).toBe(true);
  });

  it('rejects whitespace-only content', () => {
    const skill = makeSkill({ content: '   \n  ' });
    const result = validateSkill(skill);
    expect(result.valid).toBe(false);
  });

  it('collects multiple errors', () => {
    const skill = makeSkill({
      content: '',
    });
    skill.metadata.id = '';
    skill.metadata.version = 'bad';
    const result = validateSkill(skill);
    expect(result.valid).toBe(false);
    expect(result.errors.length).toBeGreaterThanOrEqual(3);
  });
});

// ---------------------------------------------------------------------------
// SkillRegistry
// ---------------------------------------------------------------------------

describe('SkillRegistry', () => {
  let registry: SkillRegistry;

  beforeEach(() => {
    registry = new SkillRegistry();
  });

  it('installs a valid skill', () => {
    registry.install(makeSkill());
    expect(registry.size).toBe(1);
  });

  it('retrieves an installed skill by id', () => {
    const skill = makeSkill();
    registry.install(skill);
    const retrieved = registry.get('test-skill');
    expect(retrieved).toBeDefined();
    expect(retrieved?.metadata.name).toBe('Test Skill');
  });

  it('returns undefined for unknown id', () => {
    expect(registry.get('nonexistent')).toBeUndefined();
  });

  it('rejects duplicate ids', () => {
    registry.install(makeSkill());
    expect(() => registry.install(makeSkill())).toThrow('already installed');
  });

  it('rejects invalid skills', () => {
    const skill = makeSkill({ content: '' });
    expect(() => registry.install(skill)).toThrow('validation failed');
  });

  it('sets installedAt on install', () => {
    registry.install(makeSkill());
    const skill = registry.get('test-skill');
    expect(skill?.installedAt).toBeDefined();
  });

  it('uninstalls a skill', () => {
    registry.install(makeSkill());
    registry.uninstall('test-skill');
    expect(registry.size).toBe(0);
    expect(registry.get('test-skill')).toBeUndefined();
  });

  it('throws on uninstalling nonexistent skill', () => {
    expect(() => registry.uninstall('nope')).toThrow('not found');
  });

  it('throws on uninstalling bundled skill', () => {
    const skill = makeSkill();
    skill.source = 'bundled';
    // Need to set a different id so it doesn't conflict
    skill.metadata.id = 'bundled-test';
    registry.install(skill);
    expect(() => registry.uninstall('bundled-test')).toThrow('bundled');
  });

  it('lists all skills', () => {
    registry.install(makeSkill());
    const s2 = makeSkill();
    s2.metadata.id = 'test-skill-2';
    s2.metadata.domain = LifeDomain.Health;
    registry.install(s2);
    expect(registry.list()).toHaveLength(2);
  });

  it('lists skills filtered by domain', () => {
    registry.install(makeSkill());
    const s2 = makeSkill();
    s2.metadata.id = 'health-skill';
    s2.metadata.domain = LifeDomain.Health;
    registry.install(s2);

    const finance = registry.list({ domain: LifeDomain.Finance });
    expect(finance).toHaveLength(1);
    expect(finance[0].metadata.id).toBe('test-skill');
  });

  it('lists skills filtered by source', () => {
    registry.install(makeSkill());
    const mp = makeSkill();
    mp.metadata.id = 'mp-skill';
    mp.source = 'marketplace';
    registry.install(mp);

    expect(registry.list({ source: 'installed' })).toHaveLength(1);
    expect(registry.list({ source: 'marketplace' })).toHaveLength(1);
  });

  it('lists skills filtered by category', () => {
    registry.install(makeSkill());
    const advisory = makeSkill();
    advisory.metadata.id = 'advisory-skill';
    advisory.metadata.category = 'advisory';
    registry.install(advisory);

    expect(registry.list({ category: 'testing' })).toHaveLength(1);
    expect(registry.list({ category: 'advisory' })).toHaveLength(1);
  });

  it('searches by name', () => {
    registry.install(makeSkill());
    expect(registry.search('Test')).toHaveLength(1);
    expect(registry.search('nonexistent')).toHaveLength(0);
  });

  it('searches by description', () => {
    registry.install(makeSkill());
    expect(registry.search('unit tests')).toHaveLength(1);
  });

  it('searches by tags', () => {
    registry.install(makeSkill());
    expect(registry.search('test')).toHaveLength(1);
  });

  it('search is case-insensitive', () => {
    registry.install(makeSkill());
    expect(registry.search('TEST SKILL')).toHaveLength(1);
  });
});

// ---------------------------------------------------------------------------
// Bundled skills
// ---------------------------------------------------------------------------

describe('bundled skills', () => {
  it('has exactly 12 bundled skills', () => {
    expect(BUNDLED_SKILLS).toHaveLength(12);
  });

  it('covers all 12 life domains', () => {
    const domains = BUNDLED_SKILLS.map((s) => s.metadata.domain);
    for (const domain of LIFE_DOMAINS) {
      expect(domains).toContain(domain);
    }
  });

  it('all bundled skills have source bundled', () => {
    for (const skill of BUNDLED_SKILLS) {
      expect(skill.source).toBe('bundled');
    }
  });

  it('all bundled skills have unique ids', () => {
    const ids = BUNDLED_SKILLS.map((s) => s.metadata.id);
    expect(new Set(ids).size).toBe(12);
  });

  it('all bundled skills pass validation', () => {
    for (const skill of BUNDLED_SKILLS) {
      // Need to create a mutable copy since BUNDLED_SKILLS is readonly
      const copy = { ...skill, metadata: { ...skill.metadata, tags: [...skill.metadata.tags], capabilities: { ...skill.metadata.capabilities, tools: [...skill.metadata.capabilities.tools], memoryScopes: [...skill.metadata.capabilities.memoryScopes] } } };
      const result = validateSkill(copy);
      expect(result.valid).toBe(true);
    }
  });

  it('all bundled skills have non-empty content', () => {
    for (const skill of BUNDLED_SKILLS) {
      expect(skill.content.length).toBeGreaterThan(0);
    }
  });

  it('getBundledSkills returns deep copies', () => {
    const skills = getBundledSkills();
    expect(skills).toHaveLength(12);
    // Mutating the copy should not affect the original
    skills[0].metadata.name = 'MUTATED';
    expect(BUNDLED_SKILLS[0].metadata.name).not.toBe('MUTATED');
  });

  it('getBundledSkillByDomain returns correct skill', () => {
    const skill = getBundledSkillByDomain('Finance');
    expect(skill).toBeDefined();
    expect(skill?.metadata.id).toBe('finance-advisor');
  });

  it('getBundledSkillByDomain returns undefined for invalid domain', () => {
    const skill = getBundledSkillByDomain('InvalidDomain');
    expect(skill).toBeUndefined();
  });

  it('getBundledSkillByDomain returns deep copy', () => {
    const skill = getBundledSkillByDomain('Finance');
    if (skill) {
      skill.metadata.name = 'MUTATED';
      expect(BUNDLED_SKILLS[0].metadata.name).not.toBe('MUTATED');
    }
  });

  it('all bundled skills have version 1.0.0', () => {
    for (const skill of BUNDLED_SKILLS) {
      expect(skill.metadata.version).toBe('1.0.0');
    }
  });
});
