/**
 * Federation package distribution.
 *
 * A FederationPackage wraps an aggregated model for distribution to devices.
 * Packages contain:
 * - Aggregated LoRA deltas (overlay).
 * - Top-K pattern bank.
 * - Updated router weights.
 *
 * Maps to rlmx-federation distribution.rs.
 */

import type { LifeDomain } from '@aix/shared';
import type { AggregatedModel } from './aggregator.js';
import { distributionFailed, packageNotFound } from './errors.js';

// ---------------------------------------------------------------------------
// FederationPackage
// ---------------------------------------------------------------------------

/** A distributable federation package containing aggregated updates. */
export interface FederationPackage {
  /** Unique identifier for this package. */
  id: string;
  /** The domain this package covers. */
  domain: LifeDomain;
  /** Which federation cycle produced this package. */
  cycleId: string;
  /** Serialized overlay LoRA delta JSON (if present). */
  overlayLora: string | null;
  /** Serialized pattern bank JSON. */
  patterns: string;
  /** Serialized router weights JSON (stub: domain confidence). */
  routerWeights: string;
  /** Semantic version of this package. */
  version: string;
  /** Size in bytes of the total package. */
  sizeBytes: number;
  /** SHA-256 checksum over all content segments. */
  checksum: string;
  /** Number of contributors that produced this package. */
  contributorCount: number;
  /** When this package was created (ISO 8601). */
  createdAt: string;
}

// ---------------------------------------------------------------------------
// SHA-256 helpers (async — uses SubtleCrypto)
// ---------------------------------------------------------------------------

async function sha256Hex(data: Uint8Array): Promise<string> {
  // Copy into a fresh ArrayBuffer to satisfy TS strict typing
  // (Uint8Array.buffer may be SharedArrayBuffer which is not assignable to BufferSource).
  const buf = new ArrayBuffer(data.length);
  new Uint8Array(buf).set(data);
  const hashBuffer = await crypto.subtle.digest('SHA-256', buf);
  return Array.from(new Uint8Array(hashBuffer))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

// ---------------------------------------------------------------------------
// Package construction
// ---------------------------------------------------------------------------

/**
 * Build a distribution package from an aggregated model.
 * Serializes patterns, LoRA delta, and router weights, then computes a
 * SHA-256 checksum over all content segments.
 */
export async function createPackageFromAggregated(
  model: AggregatedModel,
  cycleId: string,
  version: string,
): Promise<FederationPackage> {
  let patternsJson: string;
  try {
    patternsJson = JSON.stringify(model.patterns);
  } catch (e) {
    throw distributionFailed(
      `pattern serialization failed: ${e instanceof Error ? e.message : String(e)}`,
    );
  }

  let loraJson: string | null = null;
  if (model.loraDelta) {
    try {
      loraJson = JSON.stringify(model.loraDelta);
    } catch (e) {
      throw distributionFailed(
        `lora serialization failed: ${e instanceof Error ? e.message : String(e)}`,
      );
    }
  }

  let routerJson: string;
  try {
    routerJson = JSON.stringify(model.routerConfidence);
  } catch (e) {
    throw distributionFailed(
      `router weight serialization failed: ${e instanceof Error ? e.message : String(e)}`,
    );
  }

  const encoder = new TextEncoder();
  const patternsBytes = encoder.encode(patternsJson);
  const loraBytes = loraJson ? encoder.encode(loraJson) : new Uint8Array(0);
  const routerBytes = encoder.encode(routerJson);

  const sizeBytes = patternsBytes.length + loraBytes.length + routerBytes.length;

  // Compute checksum over all content segments.
  const combined = new Uint8Array(sizeBytes);
  combined.set(patternsBytes, 0);
  combined.set(loraBytes, patternsBytes.length);
  combined.set(routerBytes, patternsBytes.length + loraBytes.length);
  const checksum = await sha256Hex(combined);

  return {
    id: crypto.randomUUID(),
    domain: model.domain,
    cycleId,
    overlayLora: loraJson,
    patterns: patternsJson,
    routerWeights: routerJson,
    version,
    sizeBytes,
    checksum,
    contributorCount: model.contributorCount,
    createdAt: new Date().toISOString(),
  };
}

/**
 * Verify the integrity of a package by recomputing its checksum.
 */
export async function verifyPackageIntegrity(
  pkg: FederationPackage,
): Promise<boolean> {
  const encoder = new TextEncoder();
  const patternsBytes = encoder.encode(pkg.patterns);
  const loraBytes = pkg.overlayLora
    ? encoder.encode(pkg.overlayLora)
    : new Uint8Array(0);
  const routerBytes = encoder.encode(pkg.routerWeights);

  const totalSize = patternsBytes.length + loraBytes.length + routerBytes.length;
  const combined = new Uint8Array(totalSize);
  combined.set(patternsBytes, 0);
  combined.set(loraBytes, patternsBytes.length);
  combined.set(routerBytes, patternsBytes.length + loraBytes.length);

  const computed = await sha256Hex(combined);
  return computed === pkg.checksum;
}

// ---------------------------------------------------------------------------
// PackageDistributor
// ---------------------------------------------------------------------------

/**
 * Manages distribution of federation packages to devices.
 */
export class PackageDistributor {
  private readonly packages: FederationPackage[] = [];

  /** Publish a package for distribution. */
  publish(pkg: FederationPackage): void {
    this.packages.push(pkg);
  }

  /** Get the latest package for a given domain. */
  latestForDomain(domain: LifeDomain): FederationPackage | null {
    const domainPackages = this.packages.filter((p) => p.domain === domain);
    if (domainPackages.length === 0) return null;
    return domainPackages.reduce((latest, pkg) =>
      pkg.createdAt > latest.createdAt ? pkg : latest,
    );
  }

  /** Get a package by its ID. Throws if not found. */
  getById(id: string): FederationPackage {
    const pkg = this.packages.find((p) => p.id === id);
    if (!pkg) throw packageNotFound(id);
    return pkg;
  }

  /** List all available packages. */
  listAll(): readonly FederationPackage[] {
    return this.packages;
  }

  /** Total number of published packages. */
  get count(): number {
    return this.packages.length;
  }
}
