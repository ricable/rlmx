import type { AgentType } from '@aix/shared';
import type { AgentStatus } from './types.js';

// ---------------------------------------------------------------------------
// Discriminated union of agent domain events
// ---------------------------------------------------------------------------

export type AgentEvent =
  | AgentSpawnedEvent
  | AgentTerminatedEvent
  | AgentStatusChangedEvent
  | AgentTaskAssignedEvent
  | AgentTaskCompletedEvent
  | AgentTaskFailedEvent
  | AgentMessageSentEvent;

export interface AgentSpawnedEvent {
  readonly type: 'AgentSpawned';
  readonly agentId: string;
  readonly agentType: AgentType;
  readonly parentId?: string;
  readonly zone: string;
  readonly timestamp: Date;
}

export interface AgentTerminatedEvent {
  readonly type: 'AgentTerminated';
  readonly agentId: string;
  readonly reason?: string;
  readonly timestamp: Date;
}

export interface AgentStatusChangedEvent {
  readonly type: 'AgentStatusChanged';
  readonly agentId: string;
  readonly from: AgentStatus;
  readonly to: AgentStatus;
  readonly timestamp: Date;
}

export interface AgentTaskAssignedEvent {
  readonly type: 'AgentTaskAssigned';
  readonly agentId: string;
  readonly task: string;
  readonly timestamp: Date;
}

export interface AgentTaskCompletedEvent {
  readonly type: 'AgentTaskCompleted';
  readonly agentId: string;
  readonly task: string;
  readonly result?: unknown;
  readonly timestamp: Date;
}

export interface AgentTaskFailedEvent {
  readonly type: 'AgentTaskFailed';
  readonly agentId: string;
  readonly task: string;
  readonly error: string;
  readonly timestamp: Date;
}

export interface AgentMessageSentEvent {
  readonly type: 'AgentMessageSent';
  readonly fromId: string;
  readonly toId: string;
  readonly timestamp: Date;
}
