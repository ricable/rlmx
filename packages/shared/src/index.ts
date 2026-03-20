// @aix/shared — Shared types and utilities for the @aix ecosystem
// Re-exports everything from submodules.

export {
  SyscallPermission,
  SYSCALL_PERMISSIONS,
  LifeDomain,
  LIFE_DOMAINS,
  AgentType,
  AGENT_TYPES,
  SubscriptionTier,
  SUBSCRIPTION_TIERS,
  FederationMode,
  ResponseMode,
  VoicePersona,
} from './enums.js';

export type {
  Intent,
  GatherStrategy,
  Strategy,
  TierFeatures,
  TierLimits,
} from './types.js';

export { monthlyPriceCents, tierLimits } from './types.js';

export type {
  DomainEvent,
  DomainEventType,
  DomainEventOf,
  SyscallDispatchedEvent,
  AgentSpawnedEvent,
  QueryRoutedEvent,
  ExperimentCompletedEvent,
  NodeMembershipChangedEvent,
  StateMutatedEvent,
  VoiceSessionStartedEvent,
  IntentsDecomposedEvent,
  MeshDeviceJoinedEvent,
  FederationCycleCompletedEvent,
  ArtifactCreatedEvent,
  ArtifactBranchAdvancedEvent,
  BoardPostCreatedEvent,
  BudgetThresholdReachedEvent,
  FunctionEvolvedEvent,
  FunctionScoredEvent,
  FunctionKilledEvent,
  ApprovalRequestedEvent,
  ApprovalDecidedEvent,
  ChannelMessageReceivedEvent,
  ChannelMessageSentEvent,
} from './events.js';

export {
  DomainEventBus,
  DOMAIN_EVENT_TYPES,
  createEventBus,
} from './events.js';

export {
  AixErrorCode,
  AixError,
  capabilityDenied,
  processNotFound,
  methodNotFound,
  invalidParams,
  internalError,
  agentLimitReached,
  featureUnavailable,
  timeout,
} from './errors.js';

export type {
  JsonRpcRequest,
  JsonRpcNotification,
  JsonRpcError,
  JsonRpcSuccessResponse,
  JsonRpcErrorResponse,
  JsonRpcResponse,
} from './json-rpc.js';

export {
  createRequest,
  createNotification,
  createSuccessResponse,
  createErrorResponse,
  createErrorResponseFromAix,
  isJsonRpcRequest,
  isJsonRpcNotification,
  isSuccessResponse,
  isErrorResponse,
  parseError,
  invalidRequest,
  methodNotFoundResponse,
  invalidParamsResponse,
  internalErrorResponse,
} from './json-rpc.js';

export { generateId } from './id.js';

export type { Ok, Err, Result, AixResult } from './result.js';

export {
  ok,
  err,
  isOk,
  isErr,
  mapResult,
  mapErr,
  flatMap,
  unwrapOr,
  unwrap,
  tryAsync,
  trySync,
} from './result.js';
