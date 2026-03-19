// Re-export everything for the @aix/agents public API

// Types
export type {
  AgentStatus,
  ModelTier,
  AgentConfig,
  AgentMetrics,
  AgentInfo,
} from './types.js';

export {
  defaultMetrics,
  modelTierFor,
  preferredZone,
  maxInstances,
  canFork,
  canMutateState,
} from './types.js';

// Registry
export { AgentPermissions, PermissionRegistry } from './registry.js';

// Lifecycle
export { AgentLifecycle } from './lifecycle.js';

// Spawner
export { AgentSpawner } from './spawn.js';

// Researcher
export type {
  HypothesisStatus,
  ResearchStatus,
  Hypothesis,
  Finding,
  ResearchSummary,
  TrainingConfig,
  MutationStrategy,
} from './researcher.js';
export { ResearchObjective, ResearcherAgent } from './researcher.js';

// Events
export type {
  AgentEvent,
  AgentSpawnedEvent,
  AgentTerminatedEvent,
  AgentStatusChangedEvent,
  AgentTaskAssignedEvent,
  AgentTaskCompletedEvent,
  AgentTaskFailedEvent,
  AgentMessageSentEvent,
} from './events.js';

// Errors
export {
  AgentError,
  AgentNotFoundError,
  MaxInstancesExceededError,
  SpawnNotAllowedError,
  PermissionDeniedError,
  AlreadyTerminatedError,
  InvalidTransitionError,
  NapiBridgeUnavailableError,
} from './errors.js';
