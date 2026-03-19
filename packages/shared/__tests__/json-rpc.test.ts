import { describe, it, expect } from 'vitest';
import {
  createRequest,
  createNotification,
  createSuccessResponse,
  createErrorResponse,
  createErrorResponseFromAix,
  isJsonRpcRequest,
  isJsonRpcNotification,
  isSuccessResponse,
  isErrorResponse,
  parseError,
  invalidRequest,
  methodNotFoundResponse,
  invalidParamsResponse,
  internalErrorResponse,
} from '../src/json-rpc.js';
import { AixError, AixErrorCode } from '../src/errors.js';

describe('createRequest', () => {
  it('creates a valid JSON-RPC 2.0 request', () => {
    const req = createRequest(1, 'tools/call', { name: 'vec_search' });
    expect(req).toEqual({
      jsonrpc: '2.0',
      id: 1,
      method: 'tools/call',
      params: { name: 'vec_search' },
    });
  });

  it('omits params when undefined', () => {
    const req = createRequest('abc', 'tools/list');
    expect(req).not.toHaveProperty('params');
    expect(req.jsonrpc).toBe('2.0');
    expect(req.id).toBe('abc');
    expect(req.method).toBe('tools/list');
  });
});

describe('createNotification', () => {
  it('creates a valid notification (no id)', () => {
    const notif = createNotification('notifications/progress', { progress: 50 });
    expect(notif).toEqual({
      jsonrpc: '2.0',
      method: 'notifications/progress',
      params: { progress: 50 },
    });
    expect(notif).not.toHaveProperty('id');
  });
});

describe('createSuccessResponse', () => {
  it('creates a valid success response', () => {
    const resp = createSuccessResponse(1, { tools: [] });
    expect(resp).toEqual({
      jsonrpc: '2.0',
      id: 1,
      result: { tools: [] },
    });
  });
});

describe('createErrorResponse', () => {
  it('creates a valid error response', () => {
    const resp = createErrorResponse(1, { code: -32601, message: 'Not found' });
    expect(resp).toEqual({
      jsonrpc: '2.0',
      id: 1,
      error: { code: -32601, message: 'Not found' },
    });
  });

  it('allows null id for parse errors', () => {
    const resp = createErrorResponse(null, { code: -32700, message: 'Parse error' });
    expect(resp.id).toBeNull();
  });
});

describe('createErrorResponseFromAix', () => {
  it('converts AixError to JSON-RPC error response', () => {
    const err = new AixError(AixErrorCode.CapabilityDenied, 'no VecInsert', { perm: 'VecInsert' });
    const resp = createErrorResponseFromAix(42, err);
    expect(resp.jsonrpc).toBe('2.0');
    expect(resp.id).toBe(42);
    expect(resp.error.code).toBe(AixErrorCode.CapabilityDenied);
    expect(resp.error.message).toBe('no VecInsert');
    expect(resp.error.data).toEqual({ perm: 'VecInsert' });
  });
});

describe('type guards', () => {
  it('isJsonRpcRequest identifies requests', () => {
    expect(isJsonRpcRequest({ jsonrpc: '2.0', id: 1, method: 'foo' })).toBe(true);
    expect(isJsonRpcRequest({ jsonrpc: '2.0', method: 'foo' })).toBe(false); // notification
    expect(isJsonRpcRequest(null)).toBe(false);
    expect(isJsonRpcRequest('string')).toBe(false);
    expect(isJsonRpcRequest({ jsonrpc: '1.0', id: 1, method: 'foo' })).toBe(false);
  });

  it('isJsonRpcNotification identifies notifications', () => {
    expect(isJsonRpcNotification({ jsonrpc: '2.0', method: 'foo' })).toBe(true);
    expect(isJsonRpcNotification({ jsonrpc: '2.0', id: 1, method: 'foo' })).toBe(false);
    expect(isJsonRpcNotification(null)).toBe(false);
  });

  it('isSuccessResponse identifies success', () => {
    const success = createSuccessResponse(1, 'ok');
    const error = createErrorResponse(1, { code: -1, message: 'err' });
    expect(isSuccessResponse(success)).toBe(true);
    expect(isSuccessResponse(error)).toBe(false);
  });

  it('isErrorResponse identifies errors', () => {
    const success = createSuccessResponse(1, 'ok');
    const error = createErrorResponse(1, { code: -1, message: 'err' });
    expect(isErrorResponse(error)).toBe(true);
    expect(isErrorResponse(success)).toBe(false);
  });
});

describe('standard error response helpers', () => {
  it('parseError', () => {
    const resp = parseError(null);
    expect(resp.error.code).toBe(-32700);
    expect(resp.id).toBeNull();
  });

  it('invalidRequest', () => {
    const resp = invalidRequest(1);
    expect(resp.error.code).toBe(-32600);
  });

  it('methodNotFoundResponse', () => {
    const resp = methodNotFoundResponse(2, 'unknown_tool');
    expect(resp.error.code).toBe(-32601);
    expect(resp.error.message).toContain('unknown_tool');
  });

  it('invalidParamsResponse', () => {
    const resp = invalidParamsResponse(3, 'missing name');
    expect(resp.error.code).toBe(-32602);
    expect(resp.error.message).toBe('missing name');
  });

  it('internalErrorResponse', () => {
    const resp = internalErrorResponse(4);
    expect(resp.error.code).toBe(-32603);
  });
});
