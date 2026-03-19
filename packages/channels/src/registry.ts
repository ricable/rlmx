// Channel adapter registry (ADR-039).

import type { ChannelAdapter, ChannelMessage } from './types.js';

/** Error thrown when a channel adapter is not found. */
export class ChannelNotConfiguredError extends Error {
  constructor(name: string) {
    super(`channel adapter '${name}' not configured`);
    this.name = 'ChannelNotConfiguredError';
    Object.setPrototypeOf(this, ChannelNotConfiguredError.prototype);
  }
}

/** Registry of named channel adapters. */
export class ChannelRegistry {
  private adapters: Map<string, ChannelAdapter> = new Map();

  /** Register a named adapter. */
  register(name: string, adapter: ChannelAdapter): void {
    this.adapters.set(name, adapter);
  }

  /** Send a message through a named adapter. */
  async send(name: string, msg: ChannelMessage): Promise<void> {
    const adapter = this.adapters.get(name);
    if (!adapter) throw new ChannelNotConfiguredError(name);
    await adapter.send(msg);
  }

  /** List all registered adapter names. */
  list(): string[] {
    return Array.from(this.adapters.keys());
  }

  /** Check health of a named adapter. */
  async health(name: string): Promise<boolean> {
    const adapter = this.adapters.get(name);
    if (!adapter) throw new ChannelNotConfiguredError(name);
    return adapter.healthCheck();
  }

  /** Remove an adapter by name. Returns true if found and removed. */
  remove(name: string): boolean {
    return this.adapters.delete(name);
  }

  /** Check if an adapter is registered. */
  contains(name: string): boolean {
    return this.adapters.has(name);
  }
}
