// Trigger registry — CRUD and matching for trigger bindings (ADR-038).

import { generateId } from '@aix/shared';
import type { TriggerBinding, TriggerType } from './types.js';
import { matchEvent as matchEventFn, matchHttp as matchHttpFn, matchChannel as matchChannelFn } from './matcher.js';

/**
 * Registry of trigger bindings with matching and cycle prevention.
 */
export class TriggerRegistry {
  private bindings: TriggerBinding[] = [];
  private activeDepths: Map<string, number> = new Map();

  /** Create a new trigger binding and return its ID. */
  bind(triggerType: TriggerType, targetFunction: string, transform: string | null = null): string {
    const id = generateId();
    this.bindings.push({
      id,
      triggerType,
      targetFunction,
      transform,
      enabled: true,
      maxDepth: 5,
    });
    return id;
  }

  /** Remove a binding by ID. Returns true if found and removed. */
  unbind(id: string): boolean {
    const idx = this.bindings.findIndex((b) => b.id === id);
    if (idx === -1) return false;
    this.bindings.splice(idx, 1);
    return true;
  }

  /** Enable a binding by ID. */
  enable(id: string): boolean {
    const binding = this.bindings.find((b) => b.id === id);
    if (!binding) return false;
    binding.enabled = true;
    return true;
  }

  /** Disable a binding by ID. */
  disable(id: string): boolean {
    const binding = this.bindings.find((b) => b.id === id);
    if (!binding) return false;
    binding.enabled = false;
    return true;
  }

  /** List all bindings. */
  list(): readonly TriggerBinding[] {
    return this.bindings;
  }

  /** Get a binding by ID. */
  get(id: string): TriggerBinding | undefined {
    return this.bindings.find((b) => b.id === id);
  }

  /** Set max depth for a binding. */
  setMaxDepth(id: string, maxDepth: number): boolean {
    const binding = this.bindings.find((b) => b.id === id);
    if (!binding) return false;
    binding.maxDepth = maxDepth;
    return true;
  }

  /** Find all enabled bindings matching a domain event type. */
  matchEvent(eventType: string): TriggerBinding[] {
    return matchEventFn(this.bindings, eventType);
  }

  /** Find all enabled bindings matching an HTTP path and method. */
  matchHttp(path: string, method: string): TriggerBinding[] {
    return matchHttpFn(this.bindings, path, method);
  }

  /** Find all enabled bindings matching a channel adapter and content. */
  matchChannel(adapter: string, content: string): TriggerBinding[] {
    return matchChannelFn(this.bindings, adapter, content);
  }

  /** Check if a function can be invoked (depth < maxDepth). */
  checkDepth(functionName: string): boolean {
    const depth = this.activeDepths.get(functionName) ?? 0;
    const binding = this.bindings.find((b) => b.targetFunction === functionName);
    const maxDepth = binding?.maxDepth ?? 5;
    return depth < maxDepth;
  }

  /** Increment recursion depth for a function. */
  enterDepth(functionName: string): void {
    const current = this.activeDepths.get(functionName) ?? 0;
    this.activeDepths.set(functionName, current + 1);
  }

  /** Decrement recursion depth for a function. */
  exitDepth(functionName: string): void {
    const current = this.activeDepths.get(functionName) ?? 0;
    if (current <= 1) {
      this.activeDepths.delete(functionName);
    } else {
      this.activeDepths.set(functionName, current - 1);
    }
  }
}
