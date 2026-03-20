import { LifeDomain } from '@aix/shared';

/**
 * Built-in tag-to-LifeDomain mappings (80+ entries across all 12 domains).
 * Tags are stored and looked up in lowercase.
 */
const BUILTIN_MAPPINGS: ReadonlyArray<[string, LifeDomain]> = [
  // Domain name aliases (so 'finance' maps to Finance, etc.)
  ['finance', LifeDomain.Finance],
  ['health', LifeDomain.Health],
  ['legal', LifeDomain.Legal],
  ['career', LifeDomain.Career],
  ['education', LifeDomain.Education],
  ['home', LifeDomain.Home],
  ['shopping', LifeDomain.Shopping],
  ['travel', LifeDomain.Travel],
  ['social', LifeDomain.Social],
  ['government', LifeDomain.Government],
  ['automotive', LifeDomain.Automotive],
  ['pet', LifeDomain.Pet],

  // Finance (12)
  ['investment', LifeDomain.Finance],
  ['tax', LifeDomain.Finance],
  ['budgeting', LifeDomain.Finance],
  ['expense', LifeDomain.Finance],
  ['crypto', LifeDomain.Finance],
  ['trading', LifeDomain.Finance],
  ['banking', LifeDomain.Finance],
  ['accounting', LifeDomain.Finance],
  ['insurance-finance', LifeDomain.Finance],
  ['mortgage', LifeDomain.Finance],
  ['retirement', LifeDomain.Finance],
  ['stocks', LifeDomain.Finance],

  // Health (9)
  ['medication', LifeDomain.Health],
  ['fitness', LifeDomain.Health],
  ['nutrition', LifeDomain.Health],
  ['sleep', LifeDomain.Health],
  ['symptom', LifeDomain.Health],
  ['mental-wellness', LifeDomain.Health],
  ['medical', LifeDomain.Health],
  ['therapy', LifeDomain.Health],
  ['vaccination', LifeDomain.Health],

  // Legal (7)
  ['contract', LifeDomain.Legal],
  ['compliance', LifeDomain.Legal],
  ['rights', LifeDomain.Legal],
  ['dispute', LifeDomain.Legal],
  ['regulation', LifeDomain.Legal],
  ['patent', LifeDomain.Legal],
  ['trademark', LifeDomain.Legal],

  // Career (12)
  ['devops', LifeDomain.Career],
  ['cicd', LifeDomain.Career],
  ['monitoring', LifeDomain.Career],
  ['alerting', LifeDomain.Career],
  ['infrastructure', LifeDomain.Career],
  ['industrial', LifeDomain.Career],
  ['manufacturing', LifeDomain.Career],
  ['predictive-maintenance', LifeDomain.Career],
  ['vibration', LifeDomain.Career],
  ['quality', LifeDomain.Career],
  ['resume', LifeDomain.Career],
  ['interview', LifeDomain.Career],

  // Education (7)
  ['study', LifeDomain.Education],
  ['flashcard', LifeDomain.Education],
  ['language', LifeDomain.Education],
  ['tutoring', LifeDomain.Education],
  ['certification', LifeDomain.Education],
  ['course', LifeDomain.Education],
  ['quantum-computing', LifeDomain.Education],

  // Home — IoT, automation, agriculture, environmental (21)
  ['agriculture', LifeDomain.Home],
  ['farming', LifeDomain.Home],
  ['crops', LifeDomain.Home],
  ['irrigation', LifeDomain.Home],
  ['greenhouse', LifeDomain.Home],
  ['soil', LifeDomain.Home],
  ['livestock', LifeDomain.Home],
  ['iot', LifeDomain.Home],
  ['sensor', LifeDomain.Home],
  ['temperature', LifeDomain.Home],
  ['humidity', LifeDomain.Home],
  ['motion', LifeDomain.Home],
  ['light', LifeDomain.Home],
  ['energy-meter', LifeDomain.Home],
  ['water-leak', LifeDomain.Home],
  ['thermostat', LifeDomain.Home],
  ['lighting', LifeDomain.Home],
  ['security-cam', LifeDomain.Home],
  ['appliance', LifeDomain.Home],
  ['doorbell', LifeDomain.Home],
  ['garage', LifeDomain.Home],
  ['environmental', LifeDomain.Home],
  ['air-quality', LifeDomain.Home],
  ['water-quality', LifeDomain.Home],
  ['weather', LifeDomain.Home],
  ['uv', LifeDomain.Home],
  ['noise', LifeDomain.Home],
  ['cleaning', LifeDomain.Home],
  ['garden', LifeDomain.Home],

  // Shopping (6)
  ['shopping', LifeDomain.Shopping],
  ['price-compare', LifeDomain.Shopping],
  ['deal-finder', LifeDomain.Shopping],
  ['warranty', LifeDomain.Shopping],
  ['returns', LifeDomain.Shopping],
  ['coupon', LifeDomain.Shopping],

  // Travel (7)
  ['flight', LifeDomain.Travel],
  ['hotel', LifeDomain.Travel],
  ['itinerary', LifeDomain.Travel],
  ['currency', LifeDomain.Travel],
  ['visa', LifeDomain.Travel],
  ['packing', LifeDomain.Travel],
  ['rental-car', LifeDomain.Travel],

  // Social (7)
  ['event', LifeDomain.Social],
  ['birthday', LifeDomain.Social],
  ['gift', LifeDomain.Social],
  ['contact', LifeDomain.Social],
  ['networking', LifeDomain.Social],
  ['party', LifeDomain.Social],
  ['reunion', LifeDomain.Social],

  // Government (6)
  ['dmv', LifeDomain.Government],
  ['immigration', LifeDomain.Government],
  ['benefits', LifeDomain.Government],
  ['voter', LifeDomain.Government],
  ['permits', LifeDomain.Government],
  ['taxes-gov', LifeDomain.Government],

  // Automotive (6)
  ['maintenance', LifeDomain.Automotive],
  ['insurance-auto', LifeDomain.Automotive],
  ['fuel', LifeDomain.Automotive],
  ['diagnostics', LifeDomain.Automotive],
  ['ev-charging', LifeDomain.Automotive],
  ['parking', LifeDomain.Automotive],

  // Pet (6)
  ['feeding', LifeDomain.Pet],
  ['vet', LifeDomain.Pet],
  ['grooming', LifeDomain.Pet],
  ['training', LifeDomain.Pet],
  ['health-log', LifeDomain.Pet],
  ['pet-sitting', LifeDomain.Pet],
];

/** Pre-built map from the immutable built-in mappings, shared across instances. */
const BUILTIN_MAP: ReadonlyMap<string, LifeDomain> = new Map(BUILTIN_MAPPINGS);

export class DomainMapper {
  private mappings: Map<string, LifeDomain>;

  constructor() {
    this.mappings = new Map(BUILTIN_MAP);
  }

  /** Map a single tag to a LifeDomain. Returns undefined for unknown tags. */
  map(tag: string): LifeDomain | undefined {
    return this.mappings.get(tag.toLowerCase());
  }

  /** Map a single tag, returning fallback if not found. */
  mapOrDefault(tag: string, fallback: LifeDomain): LifeDomain {
    return this.mappings.get(tag.toLowerCase()) ?? fallback;
  }

  /** Given an array of tags, find the most common LifeDomain. */
  inferDomain(tags: string[]): LifeDomain | undefined {
    const counts = new Map<LifeDomain, number>();
    for (const tag of tags) {
      const domain = this.map(tag);
      if (domain) {
        counts.set(domain, (counts.get(domain) ?? 0) + 1);
      }
    }
    if (counts.size === 0) return undefined;

    let best: LifeDomain | undefined;
    let bestCount = 0;
    for (const [domain, count] of counts) {
      if (count > bestCount) {
        best = domain;
        bestCount = count;
      }
    }
    return best;
  }

  /** Register a custom mapping. Overwrites existing if present. */
  register(tag: string, domain: LifeDomain): void {
    this.mappings.set(tag.toLowerCase(), domain);
  }

  /** Remove a mapping. */
  unregister(tag: string): boolean {
    return this.mappings.delete(tag.toLowerCase());
  }

  /** Check if a tag has a mapping. */
  has(tag: string): boolean {
    return this.mappings.has(tag.toLowerCase());
  }

  /** Return all registered tags for a given domain. */
  tagsForDomain(domain: LifeDomain): string[] {
    const result: string[] = [];
    for (const [tag, d] of this.mappings) {
      if (d === domain) result.push(tag);
    }
    return result.sort();
  }

  /** Total number of registered mappings. */
  get size(): number {
    return this.mappings.size;
  }
}
