/**
 * New user bootstrap: load federated pattern packs for instant agent intelligence.
 *
 * When a new user installs an agent, the bootstrap process:
 * 1. Downloads the latest federated pattern pack for that domain.
 * 2. Imports anonymized patterns into the agent's SONA bank.
 * 3. Applies the aggregated LoRA delta (with EWC++ preservation).
 * 4. The agent is immediately competent -- no personal history needed.
 *
 * Maps to rlmx-federation bootstrap.rs.
 */

import type { LifeDomain } from '@aix/shared';
import type { AnonymizedPattern, LoraDelta } from './contribution.js';
import type { FederationPackage, PackageDistributor } from './distribution.js';
import { verifyPackageIntegrity } from './distribution.js';
import { bootstrapFailed } from './errors.js';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** Result of bootstrapping from a federated package. */
export interface BootstrapResult {
  /** Number of patterns imported. */
  patternsImported: number;
  /** Whether a LoRA delta was applied. */
  loraApplied: boolean;
  /** The domain that was bootstrapped. */
  domain: LifeDomain;
  /** Package version used. */
  packageVersion: string;
}

/**
 * Callback interface for importing federated data into a local SONA instance.
 *
 * The bootstrap module does not depend on a specific SONA implementation;
 * instead callers provide an adapter that records patterns and applies deltas.
 */
export interface SonaAdapter {
  /** Record a pattern into the SONA pattern bank. */
  recordPattern(action: string, actions: string[], quality: number): void;
  /**
   * Apply a LoRA delta via the adaptation mechanism.
   * Should use EWC++ regularization to preserve existing personal patterns.
   */
  applyLoraDelta(delta: LoraDelta): void;
}

// ---------------------------------------------------------------------------
// Bootstrap functions
// ---------------------------------------------------------------------------

/**
 * Bootstrap a SONA instance from the latest federated package for a domain.
 * This is the primary entry point for new-user agent bootstrapping.
 */
export async function bootstrapFromLatest(
  sona: SonaAdapter,
  distributor: PackageDistributor,
  domain: LifeDomain,
): Promise<BootstrapResult> {
  const pkg = distributor.latestForDomain(domain);
  if (!pkg) {
    throw bootstrapFailed(
      `no federated package available for domain ${domain}`,
    );
  }
  return bootstrapFromPackage(sona, pkg);
}

/**
 * Bootstrap a SONA instance from a specific federation package.
 */
export async function bootstrapFromPackage(
  sona: SonaAdapter,
  pkg: FederationPackage,
): Promise<BootstrapResult> {
  // Verify package integrity before importing.
  const valid = await verifyPackageIntegrity(pkg);
  if (!valid) {
    throw bootstrapFailed('package integrity check failed');
  }

  // Deserialize patterns.
  let patterns: AnonymizedPattern[];
  try {
    patterns = JSON.parse(pkg.patterns) as AnonymizedPattern[];
  } catch (e) {
    throw bootstrapFailed(
      `pattern deserialization failed: ${e instanceof Error ? e.message : String(e)}`,
    );
  }

  // Import patterns into SONA's pattern bank.
  const patternsImported = importPatterns(sona, patterns);

  // Apply LoRA delta if present.
  let loraApplied = false;
  if (pkg.overlayLora) {
    try {
      const delta = JSON.parse(pkg.overlayLora) as LoraDelta;
      sona.applyLoraDelta(delta);
      loraApplied = true;
    } catch {
      // Failed to apply LoRA -- continue without it.
      loraApplied = false;
    }
  }

  return {
    patternsImported,
    loraApplied,
    domain: pkg.domain,
    packageVersion: pkg.version,
  };
}

/**
 * Import anonymized patterns into a SONA pattern bank.
 * Returns the number of patterns successfully imported.
 */
function importPatterns(
  sona: SonaAdapter,
  patterns: AnonymizedPattern[],
): number {
  let imported = 0;
  for (const pattern of patterns) {
    const actionStr = pattern.actionsTaken.join(',');
    sona.recordPattern(actionStr, pattern.actionsTaken, pattern.resultQuality);
    imported++;
  }
  return imported;
}
