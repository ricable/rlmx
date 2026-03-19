/**
 * A2A JSON-RPC client — sends requests to remote A2A agents (ADR-034).
 */

import {
  generateId,
  type JsonRpcResponse,
  isSuccessResponse,
  AixError,
  AixErrorCode,
} from '@aix/shared';
import type { AgentCard, A2AMessage, A2ATask } from './types.js';

/**
 * Error thrown when an A2A request fails.
 */
export class A2AClientError extends AixError {
  constructor(message: string, details?: unknown) {
    super(AixErrorCode.InternalError, message, details);
    this.name = 'A2AClientError';
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

/** Function type for sending HTTP requests (injectable for testing). */
export type FetchFn = (
  url: string,
  init: { method: string; headers: Record<string, string>; body: string },
) => Promise<{ ok: boolean; status: number; json: () => Promise<unknown> }>;

/**
 * A2A protocol client for communicating with remote A2A agents.
 */
export class A2AClient {
  private readonly fetchFn: FetchFn;

  /**
   * @param fetchFn - Optional custom fetch function for testing.
   *                  Defaults to globalThis.fetch.
   */
  constructor(fetchFn?: FetchFn) {
    this.fetchFn = fetchFn ?? (globalThis.fetch as unknown as FetchFn);
  }

  /**
   * Discover a remote agent by fetching its agent card.
   * @param url - The base URL of the remote agent.
   */
  async discover(url: string): Promise<AgentCard> {
    const cardUrl = url.replace(/\/+$/, '') + '/.well-known/agent.json';
    const response = await this.fetchFn(cardUrl, {
      method: 'GET',
      headers: { 'Accept': 'application/json' },
      body: '',
    });

    if (!response.ok) {
      throw new A2AClientError(
        `Failed to discover agent at ${cardUrl}: HTTP ${response.status}`,
      );
    }

    return (await response.json()) as AgentCard;
  }

  /**
   * Send a message to create a new task on a remote agent.
   * @param url - The JSON-RPC endpoint of the remote agent.
   * @param message - The message to send.
   */
  async send(url: string, message: A2AMessage): Promise<A2ATask> {
    return this.rpcCall<A2ATask>(url, 'tasks/send', { message });
  }

  /**
   * Get the current state of a task on a remote agent.
   * @param url - The JSON-RPC endpoint of the remote agent.
   * @param taskId - The ID of the task to retrieve.
   */
  async getTask(url: string, taskId: string): Promise<A2ATask> {
    return this.rpcCall<A2ATask>(url, 'tasks/get', { id: taskId });
  }

  /**
   * Cancel a task on a remote agent.
   * @param url - The JSON-RPC endpoint of the remote agent.
   * @param taskId - The ID of the task to cancel.
   */
  async cancel(url: string, taskId: string): Promise<A2ATask> {
    return this.rpcCall<A2ATask>(url, 'tasks/cancel', { id: taskId });
  }

  /** Send a JSON-RPC request and extract the result. */
  private async rpcCall<T>(
    url: string,
    method: string,
    params: Record<string, unknown>,
  ): Promise<T> {
    const rpcRequest = {
      jsonrpc: '2.0' as const,
      id: generateId(),
      method,
      params,
    };

    const response = await this.fetchFn(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(rpcRequest),
    });

    if (!response.ok) {
      throw new A2AClientError(
        `HTTP error from ${url}: ${response.status}`,
      );
    }

    const rpcResponse = (await response.json()) as JsonRpcResponse<T>;

    if (!isSuccessResponse(rpcResponse)) {
      throw new A2AClientError(
        `RPC error: ${rpcResponse.error.message}`,
        rpcResponse.error,
      );
    }

    return rpcResponse.result;
  }
}
