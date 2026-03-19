// Trigger types for event-to-function bindings (ADR-038).
// Mirrors rlmx-kernel trigger.rs types.

/** HTTP trigger — fires on incoming webhook. */
export interface HttpTrigger {
  type: 'http';
  path: string;
  method: string;
}

/** Schedule trigger — fires on cron schedule. */
export interface ScheduleTrigger {
  type: 'schedule';
  cron: string;
}

/** Event trigger — fires on domain event. */
export interface EventTrigger {
  type: 'event';
  domainEvent: string;
}

/** Channel trigger — fires on channel message matching filter. */
export interface ChannelTrigger {
  type: 'channel';
  adapter: string;
  filter: string;
}

/** Discriminated union of all trigger types. */
export type TriggerType = HttpTrigger | ScheduleTrigger | EventTrigger | ChannelTrigger;

/** A binding from a trigger source to a target function. */
export interface TriggerBinding {
  /** Unique binding identifier. */
  id: string;
  /** The event source type. */
  triggerType: TriggerType;
  /** Target function name or process template. */
  targetFunction: string;
  /** Optional transform expression for input mapping. */
  transform: string | null;
  /** Whether this binding is active. */
  enabled: boolean;
  /** Maximum recursion depth (cycle prevention). */
  maxDepth: number;
}

/** Result of executing a trigger binding. */
export interface TriggerResult {
  /** The binding that was executed. */
  bindingId: string;
  /** The target function that was invoked. */
  targetFunction: string;
  /** Transformed input passed to the function. */
  input: unknown;
  /** Whether execution succeeded. */
  success: boolean;
  /** Error message if execution failed. */
  error?: string;
}
