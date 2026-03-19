import { EventEmitter } from 'node:events';

// ---------------------------------------------------------------------------
// Domain event discriminated union
// Maps 1:1 to rlmx-kernel DomainEvent enum (10 variants).
// ---------------------------------------------------------------------------

export interface SyscallDispatchedEvent {
  type: 'SyscallDispatched';
  syscallType: string;
  processId: number;
  timestamp: string;
  success: boolean;
}

export interface AgentSpawnedEvent {
  type: 'AgentSpawned';
  agentId: string;
  agentType: string;
  parentId: string | null;
  timestamp: string;
}

export interface QueryRoutedEvent {
  type: 'QueryRouted';
  queryHash: number;
  strategy: string;
  confidence: number;
  timestamp: string;
}

export interface ExperimentCompletedEvent {
  type: 'ExperimentCompleted';
  experimentId: string;
  fitness: number;
  generation: number;
  timestamp: string;
}

export interface NodeMembershipChangedEvent {
  type: 'NodeMembershipChanged';
  nodeId: string;
  joined: boolean;
  zone: string;
  timestamp: string;
}

export interface StateMutatedEvent {
  type: 'StateMutated';
  witnessId: string;
  success: boolean;
  timestamp: string;
}

export interface VoiceSessionStartedEvent {
  type: 'VoiceSessionStarted';
  sessionId: string;
  mode: string;
  timestamp: string;
}

export interface IntentsDecomposedEvent {
  type: 'IntentsDecomposed';
  sessionId: string;
  intentCount: number;
  domains: string[];
  timestamp: string;
}

export interface MeshDeviceJoinedEvent {
  type: 'MeshDeviceJoined';
  meshId: string;
  deviceId: string;
  zone: string;
  timestamp: string;
}

export interface FederationCycleCompletedEvent {
  type: 'FederationCycleCompleted';
  cycleId: string;
  contributors: number;
  patternsAggregated: number;
  timestamp: string;
}

// --- AgentOS Cherry-Pick Integration (ADR-030 through ADR-039) ---

export interface ArtifactCreatedEvent {
  type: 'ArtifactCreated';
  artifactId: string;
  creator: number;
  contentType: string;
  parentIds: string[];
  timestamp: string;
}

export interface ArtifactBranchAdvancedEvent {
  type: 'ArtifactBranchAdvanced';
  branchName: string;
  oldHead: string;
  newHead: string;
  timestamp: string;
}

export interface BoardPostCreatedEvent {
  type: 'BoardPostCreated';
  boardId: string;
  postId: string;
  author: number;
  tags: string[];
  timestamp: string;
}

export interface BudgetThresholdReachedEvent {
  type: 'BudgetThresholdReached';
  agentId: string;
  thresholdType: string;
  spent: number;
  limit: number;
  timestamp: string;
}

export interface FunctionEvolvedEvent {
  type: 'FunctionEvolved';
  functionId: string;
  name: string;
  version: number;
  status: string;
  timestamp: string;
}

export interface FunctionScoredEvent {
  type: 'FunctionScored';
  functionId: string;
  overall: number;
  correctness: number;
  safety: number;
  timestamp: string;
}

export interface FunctionKilledEvent {
  type: 'FunctionKilled';
  functionId: string;
  reason: string;
  timestamp: string;
}

export interface ApprovalRequestedEvent {
  type: 'ApprovalRequested';
  operation: string;
  agentId: string;
  tier: string;
  costEstimate: number | null;
  timestamp: string;
}

export interface ApprovalDecidedEvent {
  type: 'ApprovalDecided';
  operation: string;
  approved: boolean;
  decidedBy: string;
  timestamp: string;
}

export interface ChannelMessageReceivedEvent {
  type: 'ChannelMessageReceived';
  channelType: string;
  sender: string;
  contentPreview: string;
  timestamp: string;
}

export interface ChannelMessageSentEvent {
  type: 'ChannelMessageSent';
  channelType: string;
  recipient: string;
  timestamp: string;
}

/**
 * Discriminated union of all domain events.
 * Use the `type` field to narrow to a specific variant.
 */
export type DomainEvent =
  | SyscallDispatchedEvent
  | AgentSpawnedEvent
  | QueryRoutedEvent
  | ExperimentCompletedEvent
  | NodeMembershipChangedEvent
  | StateMutatedEvent
  | VoiceSessionStartedEvent
  | IntentsDecomposedEvent
  | MeshDeviceJoinedEvent
  | FederationCycleCompletedEvent
  | ArtifactCreatedEvent
  | ArtifactBranchAdvancedEvent
  | BoardPostCreatedEvent
  | BudgetThresholdReachedEvent
  | FunctionEvolvedEvent
  | FunctionScoredEvent
  | FunctionKilledEvent
  | ApprovalRequestedEvent
  | ApprovalDecidedEvent
  | ChannelMessageReceivedEvent
  | ChannelMessageSentEvent;

/** All valid DomainEvent type discriminators. */
export type DomainEventType = DomainEvent['type'];

/** Extract the event interface for a specific type discriminator. */
export type DomainEventOf<T extends DomainEventType> = Extract<
  DomainEvent,
  { type: T }
>;

/** All DomainEvent type strings for iteration/validation. */
export const DOMAIN_EVENT_TYPES: readonly DomainEventType[] = [
  'SyscallDispatched',
  'AgentSpawned',
  'QueryRouted',
  'ExperimentCompleted',
  'NodeMembershipChanged',
  'StateMutated',
  'VoiceSessionStarted',
  'IntentsDecomposed',
  'MeshDeviceJoined',
  'FederationCycleCompleted',
  'ArtifactCreated',
  'ArtifactBranchAdvanced',
  'BoardPostCreated',
  'BudgetThresholdReached',
  'FunctionEvolved',
  'FunctionScored',
  'FunctionKilled',
  'ApprovalRequested',
  'ApprovalDecided',
  'ChannelMessageReceived',
  'ChannelMessageSent',
] as const;

// ---------------------------------------------------------------------------
// DomainEventBus — typed EventEmitter wrapper
// ---------------------------------------------------------------------------

type EventMap = {
  [K in DomainEventType]: [DomainEventOf<K>];
};

/**
 * Typed event bus for domain events.
 * Uses Node.js EventEmitter under the hood with type-safe on/emit/off.
 */
export class DomainEventBus {
  private emitter = new EventEmitter();

  constructor(maxListeners = 1024) {
    this.emitter.setMaxListeners(maxListeners);
  }

  /** Subscribe to a specific event type. */
  on<T extends DomainEventType>(
    eventType: T,
    handler: (event: DomainEventOf<T>) => void,
  ): this {
    this.emitter.on(eventType, handler as (...args: unknown[]) => void);
    return this;
  }

  /** Subscribe to a specific event type for a single emission. */
  once<T extends DomainEventType>(
    eventType: T,
    handler: (event: DomainEventOf<T>) => void,
  ): this {
    this.emitter.once(eventType, handler as (...args: unknown[]) => void);
    return this;
  }

  /** Unsubscribe a handler from a specific event type. */
  off<T extends DomainEventType>(
    eventType: T,
    handler: (event: DomainEventOf<T>) => void,
  ): this {
    this.emitter.off(eventType, handler as (...args: unknown[]) => void);
    return this;
  }

  /** Emit an event. Returns true if the event had listeners. */
  emit(event: DomainEvent): boolean {
    return this.emitter.emit(event.type, event);
  }

  /** Subscribe to ALL domain events regardless of type. */
  onAny(handler: (event: DomainEvent) => void): this {
    for (const type of DOMAIN_EVENT_TYPES) {
      this.emitter.on(type, handler as (...args: unknown[]) => void);
    }
    return this;
  }

  /** Remove all listeners, optionally for a specific event type. */
  removeAllListeners(eventType?: DomainEventType): this {
    if (eventType) {
      this.emitter.removeAllListeners(eventType);
    } else {
      this.emitter.removeAllListeners();
    }
    return this;
  }

  /** Return the number of listeners for a given event type. */
  listenerCount(eventType: DomainEventType): number {
    return this.emitter.listenerCount(eventType);
  }
}

/**
 * Create a new DomainEventBus with default capacity.
 * Mirrors the Rust create_event_bus() helper.
 */
export function createEventBus(maxListeners = 1024): DomainEventBus {
  return new DomainEventBus(maxListeners);
}
