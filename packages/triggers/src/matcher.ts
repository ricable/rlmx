// Trigger matching functions (ADR-038).
// Stateless matching utilities that complement TriggerRegistry.

import type { TriggerBinding } from './types.js';

/** Match bindings against a domain event type. */
export function matchEvent(bindings: readonly TriggerBinding[], eventType: string): TriggerBinding[] {
  return bindings.filter(
    (b) => b.enabled && b.triggerType.type === 'event' && b.triggerType.domainEvent === eventType,
  );
}

/** Match bindings against an HTTP path and method. */
export function matchHttp(bindings: readonly TriggerBinding[], path: string, method: string): TriggerBinding[] {
  return bindings.filter(
    (b) =>
      b.enabled &&
      b.triggerType.type === 'http' &&
      b.triggerType.path === path &&
      b.triggerType.method.toLowerCase() === method.toLowerCase(),
  );
}

/** Match bindings against a channel adapter and content. */
export function matchChannel(bindings: readonly TriggerBinding[], adapter: string, content: string): TriggerBinding[] {
  return bindings.filter(
    (b) =>
      b.enabled &&
      b.triggerType.type === 'channel' &&
      b.triggerType.adapter === adapter &&
      content.includes(b.triggerType.filter),
  );
}
