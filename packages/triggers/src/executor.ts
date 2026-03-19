// Trigger executor — stub that returns the target function and transformed input (ADR-038).

import type { TriggerBinding, TriggerResult } from './types.js';

/**
 * Execute a trigger binding with the given input.
 * This is a stub implementation that returns the target function name
 * and optionally transformed input. Real execution is deferred to
 * the kernel process model.
 */
export function execute(binding: TriggerBinding, input: unknown): TriggerResult {
  let transformedInput = input;

  // Apply simple transform if specified (stub: just wraps in an object)
  if (binding.transform) {
    transformedInput = {
      _transform: binding.transform,
      _original: input,
    };
  }

  return {
    bindingId: binding.id,
    targetFunction: binding.targetFunction,
    input: transformedInput,
    success: true,
  };
}
