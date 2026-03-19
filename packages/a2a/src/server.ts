/**
 * A2A JSON-RPC server — handles tasks/send, tasks/get, tasks/cancel (ADR-034).
 */

import {
  type JsonRpcRequest,
  type JsonRpcResponse,
  createSuccessResponse,
  createErrorResponse,
  methodNotFoundResponse,
  invalidParamsResponse,
  internalErrorResponse,
  isJsonRpcRequest,
} from '@aix/shared';
import { TaskStore } from './task-store.js';
import type { A2AMessage, A2ATask, AgentCard } from './types.js';

/** Handler function invoked when a task is created or continued. */
export type TaskHandler = (task: A2ATask) => Promise<void> | void;

/**
 * A2A protocol server.
 * Handles JSON-RPC 2.0 requests for the A2A task lifecycle.
 */
export class A2AServer {
  private readonly store: TaskStore;
  private readonly card: AgentCard;
  private handler: TaskHandler | undefined;

  constructor(card: AgentCard, maxTasks?: number) {
    this.store = new TaskStore(maxTasks);
    this.card = card;
  }

  /** Register a handler that is called when tasks are created or updated. */
  onTask(handler: TaskHandler): void {
    this.handler = handler;
  }

  /** Get the agent card for this server. */
  getCard(): AgentCard {
    return this.card;
  }

  /** Handle a JSON-RPC request and return a response. */
  async handleRequest(request: unknown): Promise<JsonRpcResponse> {
    if (!isJsonRpcRequest(request)) {
      return createErrorResponse(null, {
        code: -32600,
        message: 'Invalid request',
      });
    }

    const req = request as JsonRpcRequest;

    switch (req.method) {
      case 'tasks/send':
        return this.handleSend(req);
      case 'tasks/get':
        return this.handleGet(req);
      case 'tasks/cancel':
        return this.handleCancel(req);
      default:
        return methodNotFoundResponse(req.id, req.method);
    }
  }

  private async handleSend(req: JsonRpcRequest): Promise<JsonRpcResponse> {
    const params = req.params as Record<string, unknown> | undefined;
    if (!params || !params.message) {
      return invalidParamsResponse(req.id, 'Missing required parameter: message');
    }

    const message = params.message as A2AMessage;
    const taskId = params.id as string | undefined;

    try {
      let task: A2ATask;

      if (taskId) {
        // Continue existing task
        const existing = this.store.get(taskId);
        if (!existing) {
          return createErrorResponse(req.id, {
            code: -32001,
            message: `Task not found: ${taskId}`,
          });
        }
        task = this.store.update(taskId, 'working', message);
      } else {
        // Create new task
        task = this.store.create(message);
        task = this.store.update(task.id, 'working');
      }

      if (this.handler) {
        await this.handler(task);
      }

      return createSuccessResponse(req.id, task);
    } catch (error) {
      const msg = error instanceof Error ? error.message : 'Unknown error';
      return internalErrorResponse(req.id, msg);
    }
  }

  private handleGet(req: JsonRpcRequest): JsonRpcResponse {
    const params = req.params as Record<string, unknown> | undefined;
    if (!params || typeof params.id !== 'string') {
      return invalidParamsResponse(req.id, 'Missing required parameter: id');
    }

    const task = this.store.get(params.id as string);
    if (!task) {
      return createErrorResponse(req.id, {
        code: -32001,
        message: `Task not found: ${params.id}`,
      });
    }

    return createSuccessResponse(req.id, task);
  }

  private handleCancel(req: JsonRpcRequest): JsonRpcResponse {
    const params = req.params as Record<string, unknown> | undefined;
    if (!params || typeof params.id !== 'string') {
      return invalidParamsResponse(req.id, 'Missing required parameter: id');
    }

    try {
      const task = this.store.cancel(params.id as string);
      return createSuccessResponse(req.id, task);
    } catch (error) {
      const msg = error instanceof Error ? error.message : 'Unknown error';
      return createErrorResponse(req.id, {
        code: -32001,
        message: msg,
      });
    }
  }
}
