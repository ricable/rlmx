/**
 * PluginRegistry — ports rlmx-plugin loader.rs PluginRegistry to TypeScript.
 *
 * Manages the lifecycle of domain plugins including registration,
 * unregistration, enable/disable, listing, filtering, and event emission.
 */

import type {
  DomainPlugin,
  PluginInfo,
  PluginManifest,
  PluginEvent,
  PluginEventListener,
  SafetyConstraint,
  SafetyResult,
} from './types.js';
import { PluginStatus } from './types.js';
import {
  PluginAlreadyRegisteredError,
  PluginNotFoundError,
  ManifestValidationError,
} from './errors.js';

/** Internal record for a registered plugin. */
interface PluginEntry {
  plugin: DomainPlugin;
  manifest: PluginManifest;
  status: PluginStatus;
  registeredAt: string;
}

/** Rate limit tracking entry. */
interface RateLimitEntry {
  timestamps: number[];
  maxCount: number;
  windowMs: number;
}

/**
 * Registry that holds loaded plugins and provides access to them.
 *
 * Ports the Rust PluginRegistry with additional TypeScript-idiomatic features:
 * - Manifest validation on register
 * - Enable/disable support
 * - Domain-based filtering
 * - Event emission on lifecycle changes
 * - Built-in safety engine
 */
export class PluginRegistry {
  private readonly entries: Map<string, PluginEntry> = new Map();
  private readonly listeners: Set<PluginEventListener> = new Set();
  private readonly rateLimits: Map<string, RateLimitEntry> = new Map();

  // -----------------------------------------------------------------------
  // Registration
  // -----------------------------------------------------------------------

  /**
   * Register a plugin with manifest validation.
   *
   * @param plugin - The DomainPlugin instance to register
   * @param manifest - Optional manifest override; if not provided, one is
   *   derived from the plugin's own properties
   * @throws PluginAlreadyRegisteredError if a plugin with the same name exists
   * @throws ManifestValidationError if the manifest fails validation
   */
  register(plugin: DomainPlugin, manifest?: PluginManifest): void {
    const resolved: PluginManifest = manifest ?? {
      name: plugin.name,
      version: plugin.version,
      description: plugin.description,
    };

    this.validateManifest(resolved);

    if (this.entries.has(resolved.name)) {
      throw new PluginAlreadyRegisteredError(resolved.name);
    }

    this.entries.set(resolved.name, {
      plugin,
      manifest: resolved,
      status: PluginStatus.Active,
      registeredAt: new Date().toISOString(),
    });

    this.emit({
      type: 'registered',
      pluginName: resolved.name,
      version: resolved.version,
    });
  }

  // -----------------------------------------------------------------------
  // Retrieval
  // -----------------------------------------------------------------------

  /**
   * Get a plugin by name.
   * @returns The DomainPlugin or undefined if not found.
   */
  get(name: string): DomainPlugin | undefined {
    return this.entries.get(name)?.plugin;
  }

  /**
   * Get a plugin by name, throwing if not found.
   * @throws PluginNotFoundError
   */
  getOrThrow(name: string): DomainPlugin {
    const entry = this.entries.get(name);
    if (!entry) {
      throw new PluginNotFoundError(name);
    }
    return entry.plugin;
  }

  /**
   * Check whether a plugin with the given name is registered.
   */
  has(name: string): boolean {
    return this.entries.has(name);
  }

  // -----------------------------------------------------------------------
  // Listing & filtering
  // -----------------------------------------------------------------------

  /**
   * List all registered plugins with their info.
   */
  list(): PluginInfo[] {
    return Array.from(this.entries.values()).map((entry) =>
      this.entryToInfo(entry),
    );
  }

  /**
   * List plugins filtered by domain tag.
   *
   * Only returns plugins whose manifest `domains` array contains the
   * given domain string (case-insensitive match).
   */
  listByDomain(domain: string): PluginInfo[] {
    const lowerDomain = domain.toLowerCase();
    return Array.from(this.entries.values())
      .filter((entry) =>
        entry.manifest.domains?.some((d) => d.toLowerCase() === lowerDomain),
      )
      .map((entry) => this.entryToInfo(entry));
  }

  /**
   * List only active plugins.
   */
  listActive(): PluginInfo[] {
    return Array.from(this.entries.values())
      .filter((entry) => entry.status === PluginStatus.Active)
      .map((entry) => this.entryToInfo(entry));
  }

  /**
   * Number of registered plugins.
   */
  get size(): number {
    return this.entries.size;
  }

  /**
   * Whether the registry has no plugins.
   */
  get isEmpty(): boolean {
    return this.entries.size === 0;
  }

  // -----------------------------------------------------------------------
  // Unregistration & enable/disable
  // -----------------------------------------------------------------------

  /**
   * Remove a plugin by name.
   * @returns The removed DomainPlugin, or undefined if not found.
   */
  unregister(name: string): DomainPlugin | undefined {
    const entry = this.entries.get(name);
    if (!entry) {
      return undefined;
    }
    this.entries.delete(name);
    this.emit({ type: 'unregistered', pluginName: name });
    return entry.plugin;
  }

  /**
   * Disable a plugin without removing it.
   * @throws PluginNotFoundError if the plugin is not registered.
   */
  disable(name: string): void {
    const entry = this.entries.get(name);
    if (!entry) {
      throw new PluginNotFoundError(name);
    }
    entry.status = PluginStatus.Disabled;
    this.emit({ type: 'disabled', pluginName: name });
  }

  /**
   * Re-enable a previously disabled plugin.
   * @throws PluginNotFoundError if the plugin is not registered.
   */
  enable(name: string): void {
    const entry = this.entries.get(name);
    if (!entry) {
      throw new PluginNotFoundError(name);
    }
    entry.status = PluginStatus.Active;
    this.emit({ type: 'enabled', pluginName: name });
  }

  /**
   * Get the current status of a plugin.
   * @throws PluginNotFoundError if the plugin is not registered.
   */
  getStatus(name: string): PluginStatus {
    const entry = this.entries.get(name);
    if (!entry) {
      throw new PluginNotFoundError(name);
    }
    return entry.status;
  }

  /**
   * Get the manifest of a registered plugin.
   * @throws PluginNotFoundError if the plugin is not registered.
   */
  getManifest(name: string): PluginManifest {
    const entry = this.entries.get(name);
    if (!entry) {
      throw new PluginNotFoundError(name);
    }
    return { ...entry.manifest };
  }

  // -----------------------------------------------------------------------
  // Safety engine (ported from safety.rs)
  // -----------------------------------------------------------------------

  /**
   * Check an action against a set of safety constraints.
   *
   * Returns the most restrictive result (rejected > requiresApproval > allowed).
   */
  checkSafety(
    action: string,
    params: Record<string, unknown>,
    constraints: SafetyConstraint[],
  ): SafetyResult {
    let result: SafetyResult = { status: 'allowed' };

    for (const constraint of constraints) {
      const checkResult = this.checkConstraint(action, params, constraint);
      result = this.mostRestrictive(result, checkResult);
    }

    return result;
  }

  private checkConstraint(
    action: string,
    params: Record<string, unknown>,
    constraint: SafetyConstraint,
  ): SafetyResult {
    switch (constraint.kind) {
      case 'parameterBound':
        return this.checkParameterBound(
          params,
          constraint.param,
          constraint.min,
          constraint.max,
        );
      case 'kpiGuard':
        // Placeholder: would integrate with monitoring in a real system.
        return { status: 'allowed' };
      case 'humanEscalation':
        if (constraint.actions.includes(action)) {
          return {
            status: 'requiresApproval',
            reason: `Human approval required: ${constraint.condition}`,
          };
        }
        return { status: 'allowed' };
      case 'rateLimit':
        if (action === constraint.action) {
          return this.checkRateLimit(
            constraint.action,
            constraint.maxCount,
            constraint.windowSecs,
          );
        }
        return { status: 'allowed' };
    }
  }

  private checkParameterBound(
    params: Record<string, unknown>,
    paramName: string,
    min: number,
    max: number,
  ): SafetyResult {
    const value = params[paramName];
    if (value === undefined || value === null) {
      return {
        status: 'rejected',
        reason: `Required parameter '${paramName}' is missing (expected numeric value in [${min}, ${max}])`,
      };
    }
    const num = Number(value);
    if (isNaN(num)) {
      return {
        status: 'rejected',
        reason: `Parameter '${paramName}' has non-numeric type (expected numeric value in [${min}, ${max}], got ${value})`,
      };
    }
    if (num < min || num > max) {
      return {
        status: 'rejected',
        reason: `Parameter '${paramName}' value ${num} is outside bounds [${min}, ${max}]`,
      };
    }
    return { status: 'allowed' };
  }

  private checkRateLimit(
    action: string,
    maxCount: number,
    windowSecs: number,
  ): SafetyResult {
    const now = Date.now();
    const windowMs = windowSecs * 1000;

    let entry = this.rateLimits.get(action);
    if (!entry) {
      entry = { timestamps: [], maxCount, windowMs };
      this.rateLimits.set(action, entry);
    }

    // Remove expired timestamps.
    entry.timestamps = entry.timestamps.filter((ts) => now - ts < windowMs);

    if (entry.timestamps.length >= maxCount) {
      return {
        status: 'rejected',
        reason: `Rate limit exceeded for '${action}': ${entry.timestamps.length} calls in ${windowSecs} seconds (max ${maxCount})`,
      };
    }

    entry.timestamps.push(now);
    return { status: 'allowed' };
  }

  private mostRestrictive(a: SafetyResult, b: SafetyResult): SafetyResult {
    const rank = (r: SafetyResult): number => {
      if (r.status === 'rejected') return 2;
      if (r.status === 'requiresApproval') return 1;
      return 0;
    };
    return rank(a) >= rank(b) ? a : b;
  }

  // -----------------------------------------------------------------------
  // Event system
  // -----------------------------------------------------------------------

  /**
   * Subscribe to plugin lifecycle events.
   * @returns An unsubscribe function.
   */
  on(listener: PluginEventListener): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * Remove all event listeners.
   */
  removeAllListeners(): void {
    this.listeners.clear();
  }

  private emit(event: PluginEvent): void {
    for (const listener of this.listeners) {
      try {
        listener(event);
      } catch {
        // Swallow listener errors to avoid breaking the registry.
      }
    }
  }

  // -----------------------------------------------------------------------
  // Manifest validation
  // -----------------------------------------------------------------------

  private validateManifest(manifest: PluginManifest): void {
    const violations: string[] = [];

    if (!manifest.name || manifest.name.trim().length === 0) {
      violations.push('Plugin name must not be empty');
    }

    if (manifest.name && !/^[a-z][a-z0-9-]*$/.test(manifest.name)) {
      violations.push(
        'Plugin name must be lowercase alphanumeric with hyphens, starting with a letter',
      );
    }

    if (!manifest.version || manifest.version.trim().length === 0) {
      violations.push('Plugin version must not be empty');
    }

    if (
      manifest.version &&
      !/^\d+\.\d+\.\d+/.test(manifest.version)
    ) {
      violations.push(
        'Plugin version must follow semver format (e.g., "1.0.0")',
      );
    }

    if (!manifest.description || manifest.description.trim().length === 0) {
      violations.push('Plugin description must not be empty');
    }

    if (violations.length > 0) {
      throw new ManifestValidationError(violations);
    }
  }

  // -----------------------------------------------------------------------
  // Helpers
  // -----------------------------------------------------------------------

  private entryToInfo(entry: PluginEntry): PluginInfo {
    return {
      name: entry.manifest.name,
      version: entry.manifest.version,
      description: entry.manifest.description,
      strategyCount: entry.plugin.strategyPreferences().length,
      actionCount: entry.plugin.actionExtensions().length,
      status: entry.status,
    };
  }
}
