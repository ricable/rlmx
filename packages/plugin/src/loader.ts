/**
 * Dynamic import-based plugin loading — ports rlmx-plugin loader.rs
 * dynamic/wasm modules to TypeScript.
 *
 * In TypeScript land, plugins are loaded via dynamic import() rather than
 * shared libraries or WASM modules. A plugin module must export a default
 * factory function or a `createPlugin` named export that returns a DomainPlugin.
 */

import type { DomainPlugin } from './types.js';
import { PluginLoadError } from './errors.js';

/**
 * A plugin module must export either a default function or a `createPlugin`
 * named export that returns a DomainPlugin (or a Promise of one).
 */
export interface PluginModule {
  default?: () => DomainPlugin | Promise<DomainPlugin>;
  createPlugin?: () => DomainPlugin | Promise<DomainPlugin>;
}

/**
 * Dynamically load a plugin from a module path using dynamic import().
 *
 * The module must export either:
 * - A default factory function: `export default () => new MyPlugin()`
 * - A named `createPlugin` factory: `export function createPlugin() { ... }`
 *
 * @param modulePath - Path or package name to import (passed to dynamic import)
 * @returns The loaded DomainPlugin instance
 * @throws PluginLoadError if the module cannot be loaded or has no factory
 */
export async function loadPlugin(modulePath: string): Promise<DomainPlugin> {
  let mod: PluginModule;

  try {
    mod = (await import(modulePath)) as PluginModule;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    throw new PluginLoadError(modulePath, message);
  }

  const factory = mod.default ?? mod.createPlugin;

  if (typeof factory !== 'function') {
    throw new PluginLoadError(
      modulePath,
      'Module does not export a default function or createPlugin factory',
    );
  }

  try {
    const plugin = await factory();
    return plugin;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    throw new PluginLoadError(modulePath, `Factory threw: ${message}`);
  }
}

/**
 * Load multiple plugins from module paths in parallel.
 *
 * @param modulePaths - Array of paths/package names to import
 * @returns Array of successfully loaded plugins and any errors
 */
export async function loadPlugins(modulePaths: string[]): Promise<{
  plugins: DomainPlugin[];
  errors: Array<{ path: string; error: PluginLoadError }>;
}> {
  const results = await Promise.allSettled(
    modulePaths.map(async (path) => ({
      path,
      plugin: await loadPlugin(path),
    })),
  );

  const plugins: DomainPlugin[] = [];
  const errors: Array<{ path: string; error: PluginLoadError }> = [];

  for (const result of results) {
    if (result.status === 'fulfilled') {
      plugins.push(result.value.plugin);
    } else {
      const reason = result.reason;
      if (reason instanceof PluginLoadError) {
        errors.push({
          path: (reason as PluginLoadError).message.split("'")[1] ?? 'unknown',
          error: reason,
        });
      } else {
        errors.push({
          path: 'unknown',
          error: new PluginLoadError(
            'unknown',
            reason instanceof Error ? reason.message : String(reason),
          ),
        });
      }
    }
  }

  return { plugins, errors };
}
