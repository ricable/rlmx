import { describe, it, expect } from 'vitest';
import { LifeDomain } from '@aix/shared';
import {
  getTemplates,
  getTemplate,
  getTemplatesByDomain,
  getTemplatesByOrigin,
  getTemplatesByModality,
  TEMPLATE_COUNT,
} from '../src/templates.js';
import { validateManifest } from '../src/validation.js';

describe('templates', () => {
  it('TEMPLATE_COUNT matches getTemplates() length', () => {
    expect(TEMPLATE_COUNT).toBe(getTemplates().length);
    expect(TEMPLATE_COUNT).toBeGreaterThanOrEqual(50);
  });

  it('getTemplates() returns at least 50 templates', () => {
    expect(getTemplates().length).toBeGreaterThanOrEqual(50);
  });

  it('all templates have unique IDs', () => {
    const templates = getTemplates();
    const ids = templates.map(t => t.id);
    const uniqueIds = new Set(ids);
    expect(uniqueIds.size).toBe(ids.length);
  });

  it('getTemplate by known ID returns manifest', () => {
    const manifest = getTemplate('bill-negotiator');
    expect(manifest).toBeDefined();
    expect(manifest!.id).toBe('bill-negotiator');
    expect(manifest!.name).toBe('Bill Negotiator');
  });

  it('getTemplate by unknown ID returns undefined', () => {
    expect(getTemplate('nonexistent-agent')).toBeUndefined();
  });

  it('getTemplatesByDomain(Finance) returns at least 6', () => {
    const finance = getTemplatesByDomain(LifeDomain.Finance);
    expect(finance.length).toBeGreaterThanOrEqual(6);
  });

  it('getTemplatesByDomain(Health) returns at least 6', () => {
    const health = getTemplatesByDomain(LifeDomain.Health);
    expect(health.length).toBeGreaterThanOrEqual(6);
  });

  it('getTemplatesByDomain(Home) returns at least 20 (IoT 6 + Automation 6 + Agricultural 4 + Environmental 4)', () => {
    const home = getTemplatesByDomain(LifeDomain.Home);
    expect(home.length).toBeGreaterThanOrEqual(20);
  });

  it('getTemplatesByDomain(Career) returns at least 4', () => {
    const career = getTemplatesByDomain(LifeDomain.Career);
    expect(career.length).toBeGreaterThanOrEqual(4);
  });

  it('getTemplatesByOrigin("seed") returns seed templates with correct origin', () => {
    const seedTemplates = getTemplatesByOrigin('seed');
    expect(seedTemplates.length).toBeGreaterThanOrEqual(14);
    for (const t of seedTemplates) {
      expect(t.origin.type).toBe('seed');
    }
  });

  it('getTemplatesByModality("sensor") returns sensor templates', () => {
    const sensorTemplates = getTemplatesByModality('sensor');
    expect(sensorTemplates.length).toBeGreaterThan(0);
    for (const t of sensorTemplates) {
      expect(t.modalities).toContain('sensor');
    }
  });

  it('getTemplatesByModality("voice") returns voice-enabled templates', () => {
    const voiceTemplates = getTemplatesByModality('voice');
    expect(voiceTemplates.length).toBeGreaterThan(0);
    for (const t of voiceTemplates) {
      expect(t.modalities).toContain('voice');
    }
  });

  it('all non-persistent templates validate cleanly', () => {
    const templates = getTemplates();
    for (const t of templates) {
      const result = validateManifest(t);
      // Seed sensor templates use maxRuntimeMs: 0 (persistent services) which the
      // validator flags. Filter those out -- they are valid by design.
      const nonPersistentErrors = result.errors.filter(
        e => !(e.field === 'resourceEnvelope.maxRuntimeMs' && t.resourceEnvelope.maxRuntimeMs === 0),
      );
      if (nonPersistentErrors.length > 0) {
        throw new Error(`Template "${t.id}" failed validation: ${JSON.stringify(nonPersistentErrors)}`);
      }
    }
  });

  it('templates are frozen and cannot be mutated', () => {
    const t = getTemplate('bill-negotiator');
    expect(t).toBeDefined();
    expect(Object.isFrozen(t)).toBe(true);
  });
});
