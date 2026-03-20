import type { LifeDomain } from '@aix/shared';
import type { AgentManifest, Modality } from './manifest.js';
import { duplicateManifest, manifestNotFound } from './error.js';

/** Origin type discriminant for search filtering. */
export type OriginType = 'rlmx' | 'seed' | 'external' | 'custom';

export interface ManifestSearchOptions {
  domain?: LifeDomain;
  origin?: OriginType;
  modality?: Modality;
  tag?: string;
  nameContains?: string;
}

export class ManifestRegistry {
  private manifests: Map<string, AgentManifest> = new Map();

  /** Register a manifest. Throws on duplicate id. */
  register(manifest: AgentManifest): void {
    if (this.manifests.has(manifest.id)) {
      throw duplicateManifest(manifest.id);
    }
    this.manifests.set(manifest.id, manifest);
  }

  /** Get a manifest by id. */
  get(id: string): AgentManifest | undefined {
    return this.manifests.get(id);
  }

  /** Get a manifest by id, throwing if not found. */
  getOrThrow(id: string): AgentManifest {
    const m = this.manifests.get(id);
    if (!m) throw manifestNotFound(id);
    return m;
  }

  /** Update a manifest by id. Throws if not found. */
  update(id: string, manifest: AgentManifest): void {
    if (!this.manifests.has(id)) throw manifestNotFound(id);
    this.manifests.set(id, manifest);
  }

  /** Remove a manifest by id. Returns true if it existed. */
  remove(id: string): boolean {
    return this.manifests.delete(id);
  }

  /** List all manifests. */
  list(): AgentManifest[] {
    return [...this.manifests.values()];
  }

  /** Search manifests with filters. All filters are AND-ed. Single-pass iteration. */
  search(options: ManifestSearchOptions): AgentManifest[] {
    const results: AgentManifest[] = [];
    const tagLower = options.tag?.toLowerCase();
    const nameLower = options.nameContains?.toLowerCase();

    for (const m of this.manifests.values()) {
      if (options.domain && m.lifeDomain !== options.domain) continue;
      if (options.origin && m.origin.type !== options.origin) continue;
      if (options.modality && !m.modalities.includes(options.modality)) continue;
      if (tagLower && !m.domainTags.some(t => t.toLowerCase() === tagLower)) continue;
      if (nameLower && !m.name.toLowerCase().includes(nameLower)) continue;
      results.push(m);
    }

    return results;
  }

  /** Count total manifests. */
  get size(): number {
    return this.manifests.size;
  }

  /** Check if a manifest exists. */
  has(id: string): boolean {
    return this.manifests.has(id);
  }

  /** Clear all manifests. */
  clear(): void {
    this.manifests.clear();
  }
}
