// @aix/a2a — A2A (Agent-to-Agent) protocol for the @aix ecosystem (ADR-034).

export type {
  TaskState,
  TextPart,
  FilePart,
  DataPart,
  Part,
  A2AMessage,
  A2ATask,
  A2ASkill,
  AgentCapabilities,
  AuthenticationInfo,
  AgentCard,
} from './types.js';

export { TERMINAL_STATES, canTransition } from './types.js';

export { TaskStore } from './task-store.js';

export { buildAgentCard } from './discovery.js';

export { A2AServer } from './server.js';
export type { TaskHandler } from './server.js';

export { A2AClient, A2AClientError } from './client.js';
export type { FetchFn } from './client.js';
