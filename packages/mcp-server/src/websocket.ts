// ---------------------------------------------------------------------------
// @aix/mcp-server — WebSocket event broadcasting
//
// Ports the Rust WsServer to TypeScript. Broadcasts SwarmEvent variants
// to connected clients with subscription filtering, heartbeat, backpressure,
// authentication, and connection limits.
//
// 12 SwarmEvent variants (ADR-008, ADR-018):
//   NodeJoined, NodeLeft, AgentSpawned, AgentTerminated, HealthUpdate,
//   ExperimentUpdate, MutationFound, SandboxSpawned, SandboxTerminated,
//   VoiceChunk, AgentProgress, MultimodalResponse
// ---------------------------------------------------------------------------

import { EventEmitter } from 'node:events';

// ---------------------------------------------------------------------------
// SwarmEvent types
// ---------------------------------------------------------------------------

export type Zone = 'Mac' | 'Nuc' | 'Edge' | 'Browser';
export type LeaveReason = 'Graceful' | 'Timeout' | 'Crashed';
export type TerminationReason = 'Completed' | 'Failed' | 'Cancelled' | 'ResourceLimit';
export type ExpStatus = 'Running' | 'Completed' | 'Failed' | 'CrossPollinated';
export type HapticPattern = 'Gentle' | 'DoubleTap' | 'LongBuzz' | 'Alert' | 'Success' | 'Warning';

export interface CardData {
  card_type: string;
  title: string;
  subtitle?: string;
  body?: string;
  data?: Record<string, unknown>;
  actions: string[];
  domain: string;
}

/** Discriminated union of all 12 SwarmEvent variants. */
export type SwarmEvent =
  | { type: 'NodeJoined'; payload: { node_id: string; zone: Zone; capabilities: string[] } }
  | { type: 'NodeLeft'; payload: { node_id: string; reason: LeaveReason } }
  | { type: 'AgentSpawned'; payload: { agent_id: string; agent_type: string; node_id: string } }
  | { type: 'AgentTerminated'; payload: { agent_id: string; reason: TerminationReason } }
  | { type: 'HealthUpdate'; payload: { node_id: string; cpu: number; mem_mb: number; gpu_util?: number } }
  | { type: 'ExperimentUpdate'; payload: { experiment_id: string; generation: number; val_bpb: number; status: ExpStatus } }
  | { type: 'MutationFound'; payload: { mutation_id: string; fitness: number; generation: number; parent_id?: string } }
  | { type: 'SandboxSpawned'; payload: { sandbox_id: string; profile: string; node_id?: string } }
  | { type: 'SandboxTerminated'; payload: { sandbox_id: string; reason: string } }
  | { type: 'VoiceChunk'; payload: { session_id: string; transcript: string; confidence: number; is_final: boolean } }
  | { type: 'AgentProgress'; payload: { task_id: string; agent_type: string; domain: string; progress_pct: number; status_text: string; eta_ms?: number } }
  | { type: 'MultimodalResponse'; payload: { session_id: string; voice_text?: string; visual_card?: Record<string, unknown>; haptic_pattern?: string; is_final: boolean } };

/** All event type names for default subscription. */
export const ALL_EVENT_TYPES: readonly string[] = [
  'NodeJoined',
  'NodeLeft',
  'AgentSpawned',
  'AgentTerminated',
  'HealthUpdate',
  'ExperimentUpdate',
  'MutationFound',
  'SandboxSpawned',
  'SandboxTerminated',
  'VoiceChunk',
  'AgentProgress',
  'MultimodalResponse',
] as const;

// ---------------------------------------------------------------------------
// Subscription / filter types
// ---------------------------------------------------------------------------

export interface WsSubscription {
  action: 'subscribe' | 'unsubscribe';
  event_types: string[];
  filters?: WsFilters;
}

export interface WsFilters {
  node_id?: string;
  zone?: string;
}

export interface WsAuthMessage {
  auth_token: string;
}

// ---------------------------------------------------------------------------
// Per-connection state
// ---------------------------------------------------------------------------

export class ClientState {
  eventTypes: string[];
  filters: WsFilters;

  constructor() {
    this.eventTypes = [...ALL_EVENT_TYPES];
    this.filters = {};
  }

  /** Returns true if the event matches this client's subscriptions and filters. */
  matches(event: SwarmEvent): boolean {
    // Check event type filter
    if (this.eventTypes.length > 0 && !this.eventTypes.includes(event.type)) {
      return false;
    }

    // Check node_id filter
    if (this.filters.node_id) {
      const nodeId = extractNodeId(event);
      if (nodeId && nodeId !== this.filters.node_id) {
        return false;
      }
    }

    // Check zone filter
    if (this.filters.zone) {
      const zone = extractZone(event);
      if (zone && zone.toLowerCase() !== this.filters.zone.toLowerCase()) {
        return false;
      }
    }

    return true;
  }

  subscribe(sub: WsSubscription): void {
    for (const et of sub.event_types) {
      if (!this.eventTypes.includes(et)) {
        this.eventTypes.push(et);
      }
    }
    if (sub.filters) {
      this.filters = { ...sub.filters };
    }
  }

  unsubscribe(sub: WsSubscription): void {
    this.eventTypes = this.eventTypes.filter(
      (t) => !sub.event_types.includes(t),
    );
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function extractNodeId(event: SwarmEvent): string | undefined {
  const p = event.payload as Record<string, unknown>;
  if ('node_id' in p && typeof p.node_id === 'string') return p.node_id;
  return undefined;
}

function extractZone(event: SwarmEvent): string | undefined {
  if (event.type === 'NodeJoined') return event.payload.zone;
  return undefined;
}

// ---------------------------------------------------------------------------
// SwarmEventBus
// ---------------------------------------------------------------------------

/**
 * In-process event bus for broadcasting SwarmEvents.
 * WebSocket connections subscribe to this bus and receive filtered events.
 */
export class SwarmEventBus extends EventEmitter {
  broadcast(event: SwarmEvent): void {
    this.emit('event', event);
  }

  subscribe(handler: (event: SwarmEvent) => void): () => void {
    this.on('event', handler);
    return () => this.off('event', handler);
  }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

export const MAX_WS_CONNECTIONS = 64;
export const HEARTBEAT_INTERVAL_MS = 30_000;
export const MAX_MISSED_PONGS = 3;
export const MAX_PENDING_MESSAGES = 256;
export const AUTH_TIMEOUT_MS = 5_000;

// ---------------------------------------------------------------------------
// WsServer
// ---------------------------------------------------------------------------

/**
 * WebSocket event server.
 *
 * In the TypeScript port we provide the event bus and connection management
 * primitives. The actual WebSocket upgrade and transport are handled by the
 * McpServer's HTTP layer (which can use `ws` or native Node.js WebSocket).
 *
 * This class manages:
 * - Event broadcasting with per-client filtering
 * - Client connection tracking
 * - Authentication token validation
 * - Backpressure and heartbeat configuration
 */
export class WsServer {
  readonly eventBus: SwarmEventBus;
  readonly authToken?: string;
  private _clientCount = 0;

  constructor(authToken?: string) {
    this.eventBus = new SwarmEventBus();
    this.authToken = authToken;
  }

  get clientCount(): number {
    return this._clientCount;
  }

  /** Increment connected client count. */
  addClient(): void {
    if (this._clientCount >= MAX_WS_CONNECTIONS) {
      throw new Error(
        `Connection limit reached (${MAX_WS_CONNECTIONS} max)`,
      );
    }
    this._clientCount++;
  }

  /** Decrement connected client count. */
  removeClient(): void {
    this._clientCount = Math.max(0, this._clientCount - 1);
  }

  /** Broadcast an event to all subscribers. */
  broadcast(event: SwarmEvent): void {
    this.eventBus.broadcast(event);
  }

  /** Validate an authentication token. */
  authenticate(token: string): boolean {
    if (!this.authToken) return true;
    return token === this.authToken;
  }
}
