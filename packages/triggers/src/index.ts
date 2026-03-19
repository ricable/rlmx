// @aix/triggers — Trigger primitives for event-to-function bindings (ADR-038).

export type {
  TriggerType,
  HttpTrigger,
  ScheduleTrigger,
  EventTrigger,
  ChannelTrigger,
  TriggerBinding,
  TriggerResult,
} from './types.js';

export { TriggerRegistry } from './registry.js';

export { matchEvent, matchHttp, matchChannel } from './matcher.js';

export { execute } from './executor.js';
