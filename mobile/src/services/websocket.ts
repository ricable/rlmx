/**
 * WebSocket client for the RLMX swarm event stream.
 *
 * Connects to the WS server at :3001, handles all 12 SwarmEvent variants
 * (ADR-018), and falls back to emitting simulated demo events when the
 * server is unreachable.
 */

// ---------------------------------------------------------------------------
// SwarmEvent types (mirrors crates/rlmx-mcp/src/ws.rs)
// ---------------------------------------------------------------------------

export type SwarmEventType =
  | 'NodeJoined'
  | 'NodeLeft'
  | 'AgentSpawned'
  | 'AgentTerminated'
  | 'HealthUpdate'
  | 'ExperimentUpdate'
  | 'MutationFound'
  | 'SandboxSpawned'
  | 'SandboxTerminated'
  | 'VoiceChunk'
  | 'AgentProgress'
  | 'MultimodalResponse';

export interface SwarmEvent {
  type: SwarmEventType;
  payload: Record<string, unknown>;
  timestamp: number;
}

export type SwarmEventHandler = (event: SwarmEvent) => void;

// ---------------------------------------------------------------------------
// Reconnection config
// ---------------------------------------------------------------------------

interface ReconnectConfig {
  initialDelayMs: number;
  maxDelayMs: number;
  backoffMultiplier: number;
  maxAttempts: number;
}

const DEFAULT_RECONNECT: ReconnectConfig = {
  initialDelayMs: 1000,
  maxDelayMs: 30000,
  backoffMultiplier: 2,
  maxAttempts: 10,
};

// ---------------------------------------------------------------------------
// Demo event generator
// ---------------------------------------------------------------------------

const DEMO_ZONES = ['A', 'B', 'C', 'D'] as const;
const DEMO_AGENT_TYPES = [
  'Worker', 'Researcher', 'Coordinator', 'Observer', 'Mutator', 'Validator',
] as const;

function randomDemoEvent(): SwarmEvent {
  const types: SwarmEventType[] = [
    'NodeJoined', 'NodeLeft', 'AgentSpawned', 'AgentTerminated',
    'HealthUpdate', 'ExperimentUpdate', 'MutationFound',
    'SandboxSpawned', 'SandboxTerminated', 'VoiceChunk',
    'AgentProgress', 'MultimodalResponse',
  ];
  const type = types[Math.floor(Math.random() * types.length)];
  const zone = DEMO_ZONES[Math.floor(Math.random() * DEMO_ZONES.length)];
  const nodeId = Math.floor(Math.random() * 25);
  const agentType =
    DEMO_AGENT_TYPES[Math.floor(Math.random() * DEMO_AGENT_TYPES.length)];

  const payloads: Record<SwarmEventType, Record<string, unknown>> = {
    NodeJoined: { node_id: nodeId, zone, addr: `192.168.1.${nodeId}:3001` },
    NodeLeft: { node_id: nodeId, reason: 'graceful_shutdown' },
    AgentSpawned: { node_id: nodeId, agent_type: agentType, zone },
    AgentTerminated: { agent_id: `agent-${nodeId}`, reason: 'task_complete' },
    HealthUpdate: {
      node_id: nodeId,
      cpu: Math.random() * 100,
      memory: Math.random() * 100,
      zone,
    },
    ExperimentUpdate: {
      experiment_id: `exp-${Math.floor(Math.random() * 5)}`,
      generation: Math.floor(Math.random() * 100),
      best_fitness: Math.random(),
    },
    MutationFound: {
      experiment_id: `exp-${Math.floor(Math.random() * 5)}`,
      genome_id: `genome-${Math.floor(Math.random() * 50)}`,
      fitness: Math.random(),
      improvement: Math.random() * 0.1,
    },
    SandboxSpawned: {
      sandbox_id: `sbx-${Math.floor(Math.random() * 10)}`,
      profile: 'worker_standard',
      node_id: nodeId,
    },
    SandboxTerminated: {
      sandbox_id: `sbx-${Math.floor(Math.random() * 10)}`,
      reason: 'completed',
    },
    VoiceChunk: {
      chunk_id: Math.floor(Math.random() * 1000),
      audio_format: 'pcm_16k',
      duration_ms: 250,
    },
    AgentProgress: {
      agent_id: `agent-${nodeId}`,
      domain: ['Finance', 'Health', 'Legal', 'Shopping'][
        Math.floor(Math.random() * 4)
      ],
      progress: Math.floor(Math.random() * 100),
      status: ['queued', 'working', 'done'][Math.floor(Math.random() * 3)],
    },
    MultimodalResponse: {
      request_id: `req-${Math.floor(Math.random() * 100)}`,
      modalities: ['text', 'voice'],
      text: 'Demo multimodal response',
    },
  };

  return {
    type,
    payload: payloads[type],
    timestamp: Date.now(),
  };
}

// ---------------------------------------------------------------------------
// WebSocketClient
// ---------------------------------------------------------------------------

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private url: string;
  private reconnectConfig: ReconnectConfig;
  private reconnectAttempts = 0;
  private reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
  private demoInterval: ReturnType<typeof setInterval> | null = null;
  private handlers: Map<string, Set<SwarmEventHandler>> = new Map();
  private globalHandlers: Set<SwarmEventHandler> = new Set();
  private connected = false;
  private demoMode = false;
  private intentionallyClosed = false;

  constructor(
    url = 'ws://localhost:3001',
    reconnectConfig: Partial<ReconnectConfig> = {},
  ) {
    this.url = url;
    this.reconnectConfig = { ...DEFAULT_RECONNECT, ...reconnectConfig };
  }

  // -----------------------------------------------------------------------
  // Connection lifecycle
  // -----------------------------------------------------------------------

  connect(): void {
    this.intentionallyClosed = false;
    this.attemptConnection();
  }

  disconnect(): void {
    this.intentionallyClosed = true;
    this.stopDemoMode();
    this.clearReconnectTimeout();

    if (this.ws) {
      this.ws.close(1000, 'client_disconnect');
      this.ws = null;
    }
    this.connected = false;
  }

  isConnected(): boolean {
    return this.connected;
  }

  isDemoMode(): boolean {
    return this.demoMode;
  }

  // -----------------------------------------------------------------------
  // Event subscription
  // -----------------------------------------------------------------------

  /** Subscribe to a specific event type. Returns an unsubscribe function. */
  on(eventType: SwarmEventType, handler: SwarmEventHandler): () => void {
    if (!this.handlers.has(eventType)) {
      this.handlers.set(eventType, new Set());
    }
    this.handlers.get(eventType)!.add(handler);
    return () => this.handlers.get(eventType)?.delete(handler);
  }

  /** Subscribe to all events. Returns an unsubscribe function. */
  onAny(handler: SwarmEventHandler): () => void {
    this.globalHandlers.add(handler);
    return () => this.globalHandlers.delete(handler);
  }

  // -----------------------------------------------------------------------
  // Internal
  // -----------------------------------------------------------------------

  private attemptConnection(): void {
    try {
      this.ws = new WebSocket(this.url);

      this.ws.onopen = () => {
        this.connected = true;
        this.demoMode = false;
        this.reconnectAttempts = 0;
        this.stopDemoMode();

        // Send subscription message
        this.ws?.send(
          JSON.stringify({
            type: 'subscribe',
            filter: { event_types: null, zones: null, node_ids: null },
          }),
        );
      };

      this.ws.onmessage = (event: WebSocketMessageEvent) => {
        try {
          const data = JSON.parse(String(event.data));
          // The server may send heartbeats
          if (data.type === 'heartbeat') return;

          const swarmEvent: SwarmEvent = {
            type: data.type ?? data.event_type ?? 'HealthUpdate',
            payload: data.payload ?? data,
            timestamp: data.timestamp ?? Date.now(),
          };
          this.emit(swarmEvent);
        } catch {
          // Ignore malformed messages
        }
      };

      this.ws.onerror = () => {
        // Error handling done in onclose
      };

      this.ws.onclose = () => {
        this.connected = false;
        this.ws = null;

        if (!this.intentionallyClosed) {
          this.scheduleReconnect();
        }
      };
    } catch {
      this.connected = false;
      this.startDemoMode();
    }
  }

  private emit(event: SwarmEvent): void {
    // Type-specific handlers
    const typeHandlers = this.handlers.get(event.type);
    if (typeHandlers) {
      for (const handler of typeHandlers) {
        try {
          handler(event);
        } catch {
          // Handler errors should not break the event loop
        }
      }
    }

    // Global handlers
    for (const handler of this.globalHandlers) {
      try {
        handler(event);
      } catch {
        // Handler errors should not break the event loop
      }
    }
  }

  // -----------------------------------------------------------------------
  // Reconnection with exponential backoff
  // -----------------------------------------------------------------------

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.reconnectConfig.maxAttempts) {
      this.startDemoMode();
      return;
    }

    const delay = Math.min(
      this.reconnectConfig.initialDelayMs *
        Math.pow(
          this.reconnectConfig.backoffMultiplier,
          this.reconnectAttempts,
        ),
      this.reconnectConfig.maxDelayMs,
    );

    this.reconnectAttempts += 1;
    this.reconnectTimeout = setTimeout(() => {
      this.attemptConnection();
    }, delay);
  }

  private clearReconnectTimeout(): void {
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }
  }

  // -----------------------------------------------------------------------
  // Demo mode fallback
  // -----------------------------------------------------------------------

  private startDemoMode(): void {
    if (this.demoMode) return;
    this.demoMode = true;

    // Emit demo events every 2 seconds
    this.demoInterval = setInterval(() => {
      this.emit(randomDemoEvent());
    }, 2000);
  }

  private stopDemoMode(): void {
    if (this.demoInterval) {
      clearInterval(this.demoInterval);
      this.demoInterval = null;
    }
    this.demoMode = false;
  }
}

// ---------------------------------------------------------------------------
// Singleton
// ---------------------------------------------------------------------------

let _instance: WebSocketClient | null = null;

export function getWebSocketClient(url?: string): WebSocketClient {
  if (!_instance) {
    _instance = new WebSocketClient(url);
  }
  return _instance;
}
