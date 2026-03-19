// ---------------------------------------------------------------------------
// @aix/swarm — SwarmEvent discriminated union (12 variants)
// Maps 1:1 to rlmx-swarm/src/types.rs SwarmEvent enum, plus CardData
// and HapticPattern supporting types.
// ---------------------------------------------------------------------------

import type { NodeId, ZoneId } from './types.js';

/**
 * Structured visual card data for multimodal responses (ADR-018).
 */
export interface CardData {
  cardType: string;
  title: string;
  subtitle?: string;
  body?: string;
  data?: Record<string, unknown>;
  actions: string[];
  domain: string;
}

/**
 * Haptic feedback pattern for multimodal responses (ADR-018).
 */
export enum HapticPattern {
  Gentle = 'Gentle',
  DoubleTap = 'DoubleTap',
  LongBuzz = 'LongBuzz',
  Alert = 'Alert',
  Success = 'Success',
  Warning = 'Warning',
}

// -- SwarmEvent discriminated union (12 variants) ---------------------------

export interface NodeJoinedEvent {
  type: 'NodeJoined';
  nodeId: NodeId;
  zoneId: ZoneId;
}

export interface NodeLeftEvent {
  type: 'NodeLeft';
  nodeId: NodeId;
  zoneId: ZoneId;
}

export interface HealthUpdateEvent {
  type: 'HealthUpdate';
  nodeId: NodeId;
  status: string;
}

export interface AgentSpawnedEvent {
  type: 'AgentSpawned';
  agentId: string;
  nodeId: NodeId;
}

export interface AgentTerminatedEvent {
  type: 'AgentTerminated';
  agentId: string;
  nodeId: NodeId;
}

export interface ConsensusReachedEvent {
  type: 'ConsensusReached';
  round: number;
  valueHash: string;
}

export interface ExperimentUpdateEvent {
  type: 'ExperimentUpdate';
  experimentId: string;
  progress: number;
}

export interface MutationFoundEvent {
  type: 'MutationFound';
  experimentId: string;
  description: string;
}

export interface SandboxSpawnedEvent {
  type: 'SandboxSpawned';
  sandboxId: string;
  profile: string;
  nodeId?: string;
}

export interface SandboxTerminatedEvent {
  type: 'SandboxTerminated';
  sandboxId: string;
  reason: string;
}

/**
 * Real-time voice transcription chunk (ADR-018).
 */
export interface VoiceChunkEvent {
  type: 'VoiceChunk';
  sessionId: string;
  transcript: string;
  confidence: number;
  isFinal: boolean;
}

/**
 * Per-agent progress update for multi-intent fan-out (ADR-015/018).
 */
export interface AgentProgressEvent {
  type: 'AgentProgress';
  taskId: string;
  agentType: string;
  domain: string;
  progressPct: number;
  statusText: string;
  etaMs?: number;
}

/**
 * Multimodal response combining voice, visual, and haptic channels (ADR-018).
 */
export interface MultimodalResponseEvent {
  type: 'MultimodalResponse';
  sessionId: string;
  voiceText?: string;
  visualCard?: CardData;
  hapticPattern?: HapticPattern;
  isFinal: boolean;
}

/**
 * All 12 SwarmEvent variants as a discriminated union.
 */
export type SwarmEvent =
  | NodeJoinedEvent
  | NodeLeftEvent
  | HealthUpdateEvent
  | AgentSpawnedEvent
  | AgentTerminatedEvent
  | ConsensusReachedEvent
  | ExperimentUpdateEvent
  | MutationFoundEvent
  | SandboxSpawnedEvent
  | SandboxTerminatedEvent
  | VoiceChunkEvent
  | AgentProgressEvent
  | MultimodalResponseEvent;

/**
 * All SwarmEvent type discriminators.
 */
export const SWARM_EVENT_TYPES = [
  'NodeJoined',
  'NodeLeft',
  'HealthUpdate',
  'AgentSpawned',
  'AgentTerminated',
  'ConsensusReached',
  'ExperimentUpdate',
  'MutationFound',
  'SandboxSpawned',
  'SandboxTerminated',
  'VoiceChunk',
  'AgentProgress',
  'MultimodalResponse',
] as const;

export type SwarmEventType = (typeof SWARM_EVENT_TYPES)[number];

// -- Simple event bus -------------------------------------------------------

export type SwarmEventHandler = (event: SwarmEvent) => void;

/**
 * In-process event bus for SwarmEvents. Mirrors the Rust broadcast::channel.
 */
export class SwarmEventBus {
  private handlers: SwarmEventHandler[] = [];

  /** Subscribe to all swarm events. Returns an unsubscribe function. */
  subscribe(handler: SwarmEventHandler): () => void {
    this.handlers.push(handler);
    return () => {
      const idx = this.handlers.indexOf(handler);
      if (idx >= 0) this.handlers.splice(idx, 1);
    };
  }

  /** Broadcast an event to all subscribers. */
  emit(event: SwarmEvent): void {
    for (const handler of this.handlers) {
      handler(event);
    }
  }

  /** Number of active subscribers. */
  get subscriberCount(): number {
    return this.handlers.length;
  }
}
