/** RLMX WebSocket Event Stream (ADR-008) */

export interface SwarmEvent {
  ts: string;
  type: string;
  event_type?: string;
  data?: Record<string, unknown>;
  payload?: Record<string, unknown>;
}

type EventHandler = (event: SwarmEvent) => void;

const WS_URL = 'ws://127.0.0.1:3001';
let ws: WebSocket | null = null;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
let reconnectDelay = 5000;
let reconnectAttempts = 0;
let notifiedDisconnect = false;
const handlers: Set<EventHandler> = new Set();

export function onEvent(handler: EventHandler): () => void {
  handlers.add(handler);
  return () => handlers.delete(handler);
}

export function connectWS(): void {
  if (ws?.readyState === WebSocket.OPEN || ws?.readyState === WebSocket.CONNECTING) return;
  try {
    ws = new WebSocket(WS_URL);
    ws.onopen = () => {
      reconnectDelay = 5000;
      reconnectAttempts = 0;
      notifiedDisconnect = false;
      for (const h of handlers) h({ ts: now(), type: '__connected' });
    };
    ws.onmessage = (evt) => {
      try {
        const event: SwarmEvent = { ts: now(), ...JSON.parse(evt.data) };
        for (const h of handlers) h(event);
      } catch { /* ignore parse errors */ }
    };
    ws.onclose = () => {
      // Only notify disconnect once, not on every reconnect attempt
      if (!notifiedDisconnect) {
        notifiedDisconnect = true;
        for (const h of handlers) h({ ts: now(), type: '__disconnected' });
      }
      // Exponential backoff: 5s → 10s → 20s → 30s max
      reconnectAttempts++;
      reconnectDelay = Math.min(5000 * Math.pow(1.5, reconnectAttempts), 30000);
      reconnectTimer = setTimeout(connectWS, reconnectDelay);
    };
    ws.onerror = () => { /* will trigger onclose */ };
  } catch { /* silently fail */ }
}

export function disconnectWS(): void {
  if (reconnectTimer) clearTimeout(reconnectTimer);
  reconnectTimer = null;
  ws?.close();
  ws = null;
}

export function isConnected(): boolean {
  return ws?.readyState === WebSocket.OPEN;
}

function now(): string {
  return new Date().toLocaleTimeString();
}
