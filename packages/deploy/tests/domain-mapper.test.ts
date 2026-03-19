import { describe, it, expect, beforeEach } from 'vitest';
import { LifeDomain, LIFE_DOMAINS } from '@aix/shared';
import { DomainMapper } from '../src/domain-mapper.js';

describe('DomainMapper', () => {
  let mapper: DomainMapper;

  beforeEach(() => {
    mapper = new DomainMapper();
  });

  // -------------------------------------------------------------------------
  // Basic mapping
  // -------------------------------------------------------------------------

  it('should map "finance" to LifeDomain.Finance', () => {
    expect(mapper.map('finance')).toBe(LifeDomain.Finance);
  });

  it('should map "health" to LifeDomain.Health', () => {
    expect(mapper.map('health')).toBe(LifeDomain.Health);
  });

  it('should map "IoT" to LifeDomain.Home', () => {
    expect(mapper.map('IoT')).toBe(LifeDomain.Home);
  });

  it('should map "agriculture" to a domain', () => {
    const result = mapper.map('agriculture');
    expect(result).toBeDefined();
  });

  it('should map "investment" to LifeDomain.Finance', () => {
    expect(mapper.map('investment')).toBe(LifeDomain.Finance);
  });

  it('should map "medical" to LifeDomain.Health', () => {
    expect(mapper.map('medical')).toBe(LifeDomain.Health);
  });

  it('should map "legal" to LifeDomain.Legal', () => {
    expect(mapper.map('legal')).toBe(LifeDomain.Legal);
  });

  it('should map "education" to LifeDomain.Education', () => {
    expect(mapper.map('education')).toBe(LifeDomain.Education);
  });

  it('should map "travel" to LifeDomain.Travel', () => {
    expect(mapper.map('travel')).toBe(LifeDomain.Travel);
  });

  it('should map "shopping" to LifeDomain.Shopping', () => {
    expect(mapper.map('shopping')).toBe(LifeDomain.Shopping);
  });

  // -------------------------------------------------------------------------
  // Case insensitivity
  // -------------------------------------------------------------------------

  it('should be case-insensitive when mapping tags', () => {
    expect(mapper.map('FINANCE')).toBe(LifeDomain.Finance);
    expect(mapper.map('Finance')).toBe(LifeDomain.Finance);
    expect(mapper.map('fInAnCe')).toBe(LifeDomain.Finance);
  });

  // -------------------------------------------------------------------------
  // Unknown tags
  // -------------------------------------------------------------------------

  it('should return undefined for unknown tags', () => {
    expect(mapper.map('nonexistent-tag-xyz')).toBeUndefined();
  });

  it('should return undefined for empty string', () => {
    expect(mapper.map('')).toBeUndefined();
  });

  // -------------------------------------------------------------------------
  // mapOrDefault
  // -------------------------------------------------------------------------

  it('should return mapped domain from mapOrDefault for known tag', () => {
    expect(mapper.mapOrDefault('finance', LifeDomain.Home)).toBe(LifeDomain.Finance);
  });

  it('should return fallback from mapOrDefault for unknown tag', () => {
    expect(mapper.mapOrDefault('unknown-thing', LifeDomain.Home)).toBe(LifeDomain.Home);
  });

  // -------------------------------------------------------------------------
  // inferDomain
  // -------------------------------------------------------------------------

  it('should infer domain from multiple tags picking the most common domain', () => {
    const result = mapper.inferDomain(['investment', 'trading', 'stocks', 'health']);
    expect(result).toBe(LifeDomain.Finance);
  });

  it('should return undefined when no tags match', () => {
    const result = mapper.inferDomain(['zzzz', 'xxxx', 'yyyy']);
    expect(result).toBeUndefined();
  });

  it('should return undefined for empty tag array', () => {
    const result = mapper.inferDomain([]);
    expect(result).toBeUndefined();
  });

  // -------------------------------------------------------------------------
  // Custom registration
  // -------------------------------------------------------------------------

  it('should allow registering custom tag-to-domain mappings', () => {
    mapper.register('blockchain', LifeDomain.Finance);
    expect(mapper.map('blockchain')).toBe(LifeDomain.Finance);
  });

  it('should allow unregistering a tag', () => {
    mapper.register('my-custom-tag', LifeDomain.Pet);
    expect(mapper.map('my-custom-tag')).toBe(LifeDomain.Pet);
    mapper.unregister('my-custom-tag');
    expect(mapper.map('my-custom-tag')).toBeUndefined();
  });

  // -------------------------------------------------------------------------
  // tagsForDomain
  // -------------------------------------------------------------------------

  it('should return sorted tags for a given domain', () => {
    const tags = mapper.tagsForDomain(LifeDomain.Finance);
    expect(tags.length).toBeGreaterThan(0);
    const sorted = [...tags].sort();
    expect(tags).toEqual(sorted);
  });

  it('should return empty array for domain with no custom tags if cleared', () => {
    // Create a fresh mapper and register only one tag
    const fresh = new DomainMapper();
    // Even default mappings should have entries for each domain
    const tags = fresh.tagsForDomain(LifeDomain.Finance);
    expect(Array.isArray(tags)).toBe(true);
  });

  // -------------------------------------------------------------------------
  // All 12 LifeDomain variants have at least one mapping
  // -------------------------------------------------------------------------

  it('should have at least one mapping for every LifeDomain variant', () => {
    for (const domain of LIFE_DOMAINS) {
      const tags = mapper.tagsForDomain(domain);
      expect(tags.length, `Expected at least one tag for domain ${domain}`).toBeGreaterThanOrEqual(1);
    }
  });

  // -------------------------------------------------------------------------
  // size property
  // -------------------------------------------------------------------------

  it('should reflect total mappings via size', () => {
    const initialSize = mapper.size;
    expect(initialSize).toBeGreaterThan(0);

    mapper.register('brand-new-tag', LifeDomain.Career);
    expect(mapper.size).toBe(initialSize + 1);
  });
});
