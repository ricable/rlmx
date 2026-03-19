/**
 * Codex API bridge adapter.
 *
 * Makes HTTP POST requests to an OpenAI Codex-compatible API endpoint.
 */

import type { TransportAdapter, AgentRequest, AgentResponse } from './transport.js';
import type { BridgeConfig } from './manifest.js';

/**
 * Bridge adapter that delegates inference to a Codex API endpoint.
 *
 * - `send()` makes an HTTP POST to the configured endpoint.
 * - `healthCheck()` pings the endpoint with a GET request.
 */
export class CodexBridgeAdapter implements TransportAdapter {
  private endpoint: string;
  private model: string;
  private authToken?: string;

  constructor(config: BridgeConfig) {
    this.endpoint = config.endpoint ?? 'https://api.openai.com/v1/responses';
    this.model = config.model ?? 'codex-mini-latest';
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
        input: prompt,
      });

      const controller = new AbortController();
      const timeoutId = request.timeout
        ? setTimeout(() => controller.abort(), request.timeout)
        : undefined;

      const res = await fetch(this.endpoint, {
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
      const headers: Record<string, string> = {};
      if (this.authToken) {
        headers['Authorization'] = `Bearer ${this.authToken}`;
      }

      const res = await fetch(this.endpoint.replace(/\/responses$/, '/models'), {
        method: 'GET',
        headers,
        signal: AbortSignal.timeout(10_000),
      });
      return res.ok;
    } catch {
      return false;
    }
  }

  protocol(): string {
    return 'bridge:codex';
  }
}
