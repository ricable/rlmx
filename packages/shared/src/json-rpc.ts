import { AixError, AixErrorCode } from './errors.js';

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 types
// ---------------------------------------------------------------------------

/** A JSON-RPC 2.0 request object. */
export interface JsonRpcRequest<P = unknown> {
  jsonrpc: '2.0';
  id: string | number;
  method: string;
  params?: P;
}

/** A JSON-RPC 2.0 notification (no id, no response expected). */
export interface JsonRpcNotification<P = unknown> {
  jsonrpc: '2.0';
  method: string;
  params?: P;
}

/** A JSON-RPC 2.0 error object embedded in a response. */
export interface JsonRpcError {
  code: number;
  message: string;
  data?: unknown;
}

/** A successful JSON-RPC 2.0 response. */
export interface JsonRpcSuccessResponse<R = unknown> {
  jsonrpc: '2.0';
  id: string | number;
  result: R;
}

/** An error JSON-RPC 2.0 response. */
export interface JsonRpcErrorResponse {
  jsonrpc: '2.0';
  id: string | number | null;
  error: JsonRpcError;
}

/** A JSON-RPC 2.0 response (success or error). */
export type JsonRpcResponse<R = unknown> =
  | JsonRpcSuccessResponse<R>
  | JsonRpcErrorResponse;

// ---------------------------------------------------------------------------
// Builder helpers
// ---------------------------------------------------------------------------

/** Create a JSON-RPC 2.0 request. */
export function createRequest<P>(
  id: string | number,
  method: string,
  params?: P,
): JsonRpcRequest<P> {
  return {
    jsonrpc: '2.0',
    id,
    method,
    ...(params !== undefined ? { params } : {}),
  };
}

/** Create a JSON-RPC 2.0 notification (no response expected). */
export function createNotification<P>(
  method: string,
  params?: P,
): JsonRpcNotification<P> {
  return {
    jsonrpc: '2.0',
    method,
    ...(params !== undefined ? { params } : {}),
  };
}

/** Create a successful JSON-RPC 2.0 response. */
export function createSuccessResponse<R>(
  id: string | number,
  result: R,
): JsonRpcSuccessResponse<R> {
  return { jsonrpc: '2.0', id, result };
}

/** Create an error JSON-RPC 2.0 response. */
export function createErrorResponse(
  id: string | number | null,
  error: JsonRpcError,
): JsonRpcErrorResponse {
  return { jsonrpc: '2.0', id, error };
}

/** Create an error JSON-RPC 2.0 response from an AixError. */
export function createErrorResponseFromAix(
  id: string | number | null,
  err: AixError,
): JsonRpcErrorResponse {
  return createErrorResponse(id, err.toJSON());
}

// ---------------------------------------------------------------------------
// Type guards
// ---------------------------------------------------------------------------

/** Check whether a value is a JSON-RPC 2.0 request. */
export function isJsonRpcRequest(value: unknown): value is JsonRpcRequest {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    obj.jsonrpc === '2.0' &&
    typeof obj.method === 'string' &&
    ('id' in obj)
  );
}

/** Check whether a value is a JSON-RPC 2.0 notification. */
export function isJsonRpcNotification(
  value: unknown,
): value is JsonRpcNotification {
  if (typeof value !== 'object' || value === null) return false;
  const obj = value as Record<string, unknown>;
  return (
    obj.jsonrpc === '2.0' &&
    typeof obj.method === 'string' &&
    !('id' in obj)
  );
}

/** Check whether a JSON-RPC response is a success response. */
export function isSuccessResponse<R>(
  response: JsonRpcResponse<R>,
): response is JsonRpcSuccessResponse<R> {
  return 'result' in response;
}

/** Check whether a JSON-RPC response is an error response. */
export function isErrorResponse(
  response: JsonRpcResponse,
): response is JsonRpcErrorResponse {
  return 'error' in response;
}

// ---------------------------------------------------------------------------
// Standard error responses
// ---------------------------------------------------------------------------

export function parseError(
  id: string | number | null,
  message = 'Parse error',
): JsonRpcErrorResponse {
  return createErrorResponse(id, {
    code: AixErrorCode.ParseError,
    message,
  });
}

export function invalidRequest(
  id: string | number | null,
  message = 'Invalid request',
): JsonRpcErrorResponse {
  return createErrorResponse(id, {
    code: AixErrorCode.InvalidRequest,
    message,
  });
}

export function methodNotFoundResponse(
  id: string | number | null,
  method: string,
): JsonRpcErrorResponse {
  return createErrorResponse(id, {
    code: AixErrorCode.MethodNotFound,
    message: `Method not found: ${method}`,
  });
}

export function invalidParamsResponse(
  id: string | number | null,
  message: string,
): JsonRpcErrorResponse {
  return createErrorResponse(id, {
    code: AixErrorCode.InvalidParams,
    message,
  });
}

export function internalErrorResponse(
  id: string | number | null,
  message = 'Internal error',
): JsonRpcErrorResponse {
  return createErrorResponse(id, {
    code: AixErrorCode.InternalError,
    message,
  });
}
