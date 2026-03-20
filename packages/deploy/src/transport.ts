import type { JsonRpcResponse } from '@aix/shared';
import { createRequest, isSuccessResponse } from '@aix/shared';
import type { TransportSpec, BridgeConfig } from './manifest.js';
import { ClaudeCodeBridgeAdapter } from './bridge-claude-code.js';
import { CodexBridgeAdapter } from './bridge-codex.js';
import { HttpGenericBridgeAdapter } from './bridge-http-generic.js';

// --- Request/Response types ---
export interface AgentRequest {
  method: string;
  params?: unknown;
  timeout?: number;
}

export interface AgentResponse {
  status: 'ok' | 'error' | 'unavailable';
  data?: unknown;
  error?: string;
}

// --- Transport Adapter interface ---
export interface TransportAdapter {
  send(request: AgentRequest): Promise<AgentResponse>;
  healthCheck(): Promise<boolean>;
  protocol(): string;
}

// --- MCP Transport (JSON-RPC 2.0 over HTTP) ---
export class McpTransportAdapter implements TransportAdapter {
  private endpoint: string;
  private headers: Record<string, string>;
  private requestId = 0;

  constructor(endpoint: string, options?: { authToken?: string; headers?: Record<string, string> }) {
    this.endpoint = endpoint;
    this.headers = {
      'Content-Type': 'application/json',
      ...options?.headers,
    };
    if (options?.authToken) {
      this.headers['Authorization'] = `Bearer ${options.authToken}`;
    }
  }

  async send(request: AgentRequest): Promise<AgentResponse> {
    const rpcRequest = createRequest(++this.requestId, request.method, request.params);
    try {
      const controller = new AbortController();
      const timeoutId = request.timeout ? setTimeout(() => controller.abort(), request.timeout) : undefined;
      const res = await fetch(this.endpoint, {
        method: 'POST',
        headers: this.headers,
        body: JSON.stringify(rpcRequest),
        signal: controller.signal,
      });
      if (timeoutId) clearTimeout(timeoutId);

      if (!res.ok) {
        return { status: 'error', error: `HTTP ${res.status}: ${res.statusText}` };
      }

      const body = await res.json() as JsonRpcResponse;
      if (isSuccessResponse(body)) {
        return { status: 'ok', data: body.result };
      }
      return { status: 'error', error: body.error.message };
    } catch (e) {
      return { status: 'error', error: e instanceof Error ? e.message : String(e) };
    }
  }

  async healthCheck(): Promise<boolean> {
    try {
      const res = await this.send({ method: 'ping', timeout: 5000 });
      return res.status === 'ok';
    } catch {
      return false;
    }
  }

  protocol(): string { return 'mcp'; }
}

// --- REST Transport ---
export class RestTransportAdapter implements TransportAdapter {
  private baseUrl: string;
  private headers: Record<string, string>;

  constructor(baseUrl: string, options?: { authToken?: string; headers?: Record<string, string> }) {
    this.baseUrl = baseUrl.replace(/\/$/, '');
    this.headers = {
      'Content-Type': 'application/json',
      ...options?.headers,
    };
    if (options?.authToken) {
      this.headers['Authorization'] = `Bearer ${options.authToken}`;
    }
  }

  async send(request: AgentRequest): Promise<AgentResponse> {
    try {
      const controller = new AbortController();
      const timeoutId = request.timeout ? setTimeout(() => controller.abort(), request.timeout) : undefined;
      const url = `${this.baseUrl}/${request.method.replace(/\./g, '/')}`;
      const res = await fetch(url, {
        method: 'POST',
        headers: this.headers,
        body: request.params ? JSON.stringify(request.params) : undefined,
        signal: controller.signal,
      });
      if (timeoutId) clearTimeout(timeoutId);

      if (!res.ok) {
        if (res.status === 429) {
          return { status: 'error', error: 'rate_limited' };
        }
        return { status: 'error', error: `HTTP ${res.status}: ${res.statusText}` };
      }

      const data = await res.json();
      return { status: 'ok', data };
    } catch (e) {
      return { status: 'error', error: e instanceof Error ? e.message : String(e) };
    }
  }

  async healthCheck(): Promise<boolean> {
    try {
      const res = await fetch(`${this.baseUrl}/health`, { headers: this.headers });
      return res.ok;
    } catch {
      return false;
    }
  }

  protocol(): string { return 'rest'; }
}

// --- WebSocket Transport ---
export class WebSocketTransportAdapter implements TransportAdapter {
  private url: string;
  private ws: WebSocket | null = null;
  private pending: Map<number, { resolve: (r: AgentResponse) => void; timer?: ReturnType<typeof setTimeout> }> = new Map();
  private requestId = 0;

  constructor(url: string) {
    this.url = url;
  }

  private static CONNECTION_TIMEOUT_MS = 10_000;
  private connectingPromise: Promise<WebSocket> | null = null;

  private async ensureConnection(): Promise<WebSocket> {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) return this.ws;
    if (this.connectingPromise) return this.connectingPromise;

    this.connectingPromise = new Promise<WebSocket>((resolve, reject) => {
      const ws = new WebSocket(this.url);
      const timer = setTimeout(() => {
        ws.close();
        reject(new Error('WebSocket connection timeout'));
      }, WebSocketTransportAdapter.CONNECTION_TIMEOUT_MS);

      ws.onopen = () => {
        clearTimeout(timer);
        this.ws = ws;
        this.connectingPromise = null;
        resolve(ws);
      };
      ws.onerror = (_e) => {
        clearTimeout(timer);
        this.connectingPromise = null;
        reject(new Error('WebSocket connection failed'));
      };
      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data as string);
          if (data.id && this.pending.has(data.id)) {
            const entry = this.pending.get(data.id)!;
            if (entry.timer) clearTimeout(entry.timer);
            this.pending.delete(data.id);
            if (data.error) {
              entry.resolve({ status: 'error', error: data.error.message ?? String(data.error) });
            } else {
              entry.resolve({ status: 'ok', data: data.result });
            }
          }
        } catch { /* ignore non-JSON messages */ }
      };
      ws.onclose = () => {
        this.ws = null;
        // Reject all pending requests
        for (const [_id, entry] of this.pending) {
          if (entry.timer) clearTimeout(entry.timer);
          entry.resolve({ status: 'error', error: 'WebSocket closed' });
        }
        this.pending.clear();
      };
    });
    return this.connectingPromise;
  }

  async send(request: AgentRequest): Promise<AgentResponse> {
    try {
      const ws = await this.ensureConnection();
      const id = ++this.requestId;
      const rpcRequest = createRequest(id, request.method, request.params);

      if (ws.readyState !== WebSocket.OPEN) {
        return { status: 'error', error: 'WebSocket not open' };
      }

      return new Promise((resolve) => {
        const timer = request.timeout
          ? setTimeout(() => {
              this.pending.delete(id);
              resolve({ status: 'error', error: 'timeout' });
            }, request.timeout)
          : undefined;

        this.pending.set(id, { resolve, timer });
        ws.send(JSON.stringify(rpcRequest));
      });
    } catch (e) {
      return { status: 'error', error: e instanceof Error ? e.message : String(e) };
    }
  }

  async healthCheck(): Promise<boolean> {
    try {
      await this.ensureConnection();
      return true;
    } catch {
      return false;
    }
  }

  protocol(): string { return 'ws'; }

  close(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}

// --- Stub Transport (returns unavailable) ---
export class StubTransportAdapter implements TransportAdapter {
  private proto: string;

  constructor(protocol: string = 'stub') {
    this.proto = protocol;
  }

  async send(_request: AgentRequest): Promise<AgentResponse> {
    return { status: 'unavailable' };
  }

  async healthCheck(): Promise<boolean> {
    return false;
  }

  protocol(): string { return this.proto; }
}

// --- Factory ---
export function createTransportAdapter(spec: TransportSpec): TransportAdapter {
  switch (spec.type) {
    case 'mcp':
      return new McpTransportAdapter(spec.endpoint);
    case 'rest':
      return new RestTransportAdapter(spec.baseUrl);
    case 'ws':
      return new WebSocketTransportAdapter(spec.url);
    case 'broadcast-channel':
    case 'quic':
    case 'mqtt':
    case 'grpc':
      return new StubTransportAdapter(spec.type);
    case 'bridge': {
      const config = spec.config;
      switch (spec.runtime) {
        case 'claude-code':
          return new ClaudeCodeBridgeAdapter(config);
        case 'codex':
          return new CodexBridgeAdapter(config);
        case 'cursor':
          return new StubTransportAdapter('bridge:cursor');
        case 'opencode':
          return new StubTransportAdapter('bridge:opencode');
        case 'http-generic':
          return new HttpGenericBridgeAdapter(config);
      }
    }
  }
}
