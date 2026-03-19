/**
 * Generic HTTP bridge adapter.
 *
 * Makes HTTP POST requests to any OpenAI-compatible API endpoint.
 */

import type { TransportAdapter, AgentRequest, AgentResponse } from './transport.js';
import type { BridgeConfig } from './manifest.js';

/**
 * Bridge adapter for generic HTTP LLM endpoints (OpenAI-compatible format).
 *
 * - `send()` makes an HTTP POST to `{endpoint}/chat/completions`.
 * - `healthCheck()` checks `/health` or `/v1/models`.
 */
export class HttpGenericBridgeAdapter implements TransportAdapter {
  private endpoint: string;
  private model: string;
  private authToken?: string;

  constructor(config: BridgeConfig) {
    this.endpoint = (config.endpoint ?? 'http://localhost:8080').replace(/\/$/, '');
    this.model = config.model ?? 'default';
    if (config.authEnvVar) {
      this.authToken = process.env[config.authEnvVar];
    }
  }

  async send(request: AgentRequest): Promise<AgentResponse> {
    try {
      const prompt = typeof request.params === 'string'
        ? request.params
        : JSON.stringify(request.params ?? request.method);

      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
      };
      if (this.authToken) {
        headers['Authorization'] = `Bearer ${this.authToken}`;
      }

      const body = JSON.stringify({
        model: this.model,
        messages: [{ role: 'user', content: prompt }],
      });

      const url = `${this.endpoint}/v1/chat/completions`;

      const controller = new AbortController();
      const timeoutId = request.timeout
        ? setTimeout(() => controller.abort(), request.timeout)
        : undefined;

      const res = await fetch(url, {
        method: 'POST',
        headers,
        body,
        signal: controller.signal,
      });

      if (timeoutId) clearTimeout(timeoutId);

      if (!res.ok) {
        return { status: 'error', error: `HTTP ${res.status}: ${res.statusText}` };
      }

      const data = await res.json();
      return { status: 'ok', data };
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      return { status: 'error', error: message };
    }
  }

  async healthCheck(): Promise<boolean> {
    try {
      // Try /health first, fall back to /v1/models
      const healthUrl = `${this.endpoint}/health`;
      const headers: Record<string, string> = {};
      if (this.authToken) {
        headers['Authorization'] = `Bearer ${this.authToken}`;
      }

      const res = await fetch(healthUrl, {
        method: 'GET',
        headers,
        signal: AbortSignal.timeout(10_000),
      });

      if (res.ok) return true;

      // Fallback: try /v1/models
      const modelsUrl = `${this.endpoint}/v1/models`;
      const res2 = await fetch(modelsUrl, {
        method: 'GET',
        headers,
        signal: AbortSignal.timeout(10_000),
      });
      return res2.ok;
    } catch {
      return false;
    }
  }

  protocol(): string {
    return 'bridge:http-generic';
  }
}
