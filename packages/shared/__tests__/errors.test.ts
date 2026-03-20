import { describe, it, expect } from 'vitest';
import {
  AixError,
  AixErrorCode,
  capabilityDenied,
  processNotFound,
  methodNotFound,
  invalidParams,
  internalError,
  agentLimitReached,
  featureUnavailable,
  timeout,
} from '../src/errors.js';

describe('AixError', () => {
  it('extends Error', () => {
    const err = new AixError(AixErrorCode.InternalError, 'test');
    expect(err).toBeInstanceOf(Error);
    expect(err).toBeInstanceOf(AixError);
  });

  it('has correct name', () => {
    const err = new AixError(AixErrorCode.InternalError, 'test');
    expect(err.name).toBe('AixError');
  });

  it('preserves code, message, and details', () => {
    const err = new AixError(AixErrorCode.CapabilityDenied, 'denied', { perm: 'VecInsert' });
    expect(err.code).toBe(AixErrorCode.CapabilityDenied);
    expect(err.message).toBe('denied');
    expect(err.details).toEqual({ perm: 'VecInsert' });
  });

  it('toJSON serializes correctly without details', () => {
    const err = new AixError(AixErrorCode.Timeout, 'timed out');
    const json = err.toJSON();
    expect(json).toEqual({
      code: AixErrorCode.Timeout,
      message: 'timed out',
    });
    expect(json).not.toHaveProperty('data');
  });

  it('toJSON serializes correctly with details', () => {
    const err = new AixError(AixErrorCode.InvalidParams, 'bad input', { field: 'name' });
    const json = err.toJSON();
    expect(json).toEqual({
      code: AixErrorCode.InvalidParams,
      message: 'bad input',
      data: { field: 'name' },
    });
  });
});

describe('AixErrorCode', () => {
  it('JSON-RPC standard codes are in -32xxx range', () => {
    expect(AixErrorCode.ParseError).toBe(-32700);
    expect(AixErrorCode.InvalidRequest).toBe(-32600);
    expect(AixErrorCode.MethodNotFound).toBe(-32601);
    expect(AixErrorCode.InvalidParams).toBe(-32602);
    expect(AixErrorCode.InternalError).toBe(-32603);
  });

  it('kernel codes are in -33xxx range', () => {
    expect(AixErrorCode.CapabilityDenied).toBe(-33001);
    expect(AixErrorCode.Timeout).toBe(-33007);
  });

  it('agent codes are in -34xxx range', () => {
    expect(AixErrorCode.AgentNotFound).toBe(-34001);
    expect(AixErrorCode.AgentAlreadyTerminated).toBe(-34005);
  });

  it('billing codes are in -35xxx range', () => {
    expect(AixErrorCode.AgentLimitReached).toBe(-35001);
    expect(AixErrorCode.FeatureUnavailable).toBe(-35003);
  });
});

describe('error factory helpers', () => {
  it('capabilityDenied', () => {
    const err = capabilityDenied('VecInsert');
    expect(err.code).toBe(AixErrorCode.CapabilityDenied);
    expect(err.message).toContain('VecInsert');
  });

  it('processNotFound', () => {
    const err = processNotFound('abc-123');
    expect(err.code).toBe(AixErrorCode.ProcessNotFound);
    expect(err.message).toContain('abc-123');
  });

  it('methodNotFound', () => {
    const err = methodNotFound('unknown_tool');
    expect(err.code).toBe(AixErrorCode.MethodNotFound);
    expect(err.message).toContain('unknown_tool');
  });

  it('invalidParams', () => {
    const err = invalidParams('missing field', { field: 'name' });
    expect(err.code).toBe(AixErrorCode.InvalidParams);
    expect(err.details).toEqual({ field: 'name' });
  });

  it('internalError', () => {
    const err = internalError('something broke');
    expect(err.code).toBe(AixErrorCode.InternalError);
  });

  it('agentLimitReached', () => {
    const err = agentLimitReached(5, 5);
    expect(err.code).toBe(AixErrorCode.AgentLimitReached);
    expect(err.message).toContain('5/5');
  });

  it('featureUnavailable', () => {
    const err = featureUnavailable('custom_sdk');
    expect(err.code).toBe(AixErrorCode.FeatureUnavailable);
    expect(err.message).toContain('custom_sdk');
  });

  it('timeout', () => {
    const err = timeout(30000);
    expect(err.code).toBe(AixErrorCode.Timeout);
    expect(err.message).toContain('30000');
  });
});
