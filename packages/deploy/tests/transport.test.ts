import { describe, it, expect } from 'vitest';
import {
  StubTransportAdapter,
  McpTransportAdapter,
  RestTransportAdapter,
  WebSocketTransportAdapter,
  createTransportAdapter,
} from '../src/transport.js';
import type { TransportAdapter, AgentRequest, AgentResponse } from '../src/transport.js';

describe('StubTransportAdapter', () => {
  it('send() always returns { status: "unavailable" }', async () => {
    const stub = new StubTransportAdapter();
    const res = await stub.send({ method: 'anything' });
    expect(res).toEqual({ status: 'unavailable' });
  });

  it('send() returns unavailable regardless of method or params', async () => {
    const stub = new StubTransportAdapter('custom');
    const res = await stub.send({ method: 'foo.bar', params: { x: 1 }, timeout: 5000 });
    expect(res.status).toBe('unavailable');
  });

  it('healthCheck() returns false', async () => {
    const stub = new StubTransportAdapter();
    expect(await stub.healthCheck()).toBe(false);
  });

  it('protocol() returns default "stub"', () => {
    const stub = new StubTransportAdapter();
    expect(stub.protocol()).toBe('stub');
  });

  it('protocol() returns configured protocol string', () => {
    const stub = new StubTransportAdapter('custom-proto');
    expect(stub.protocol()).toBe('custom-proto');
  });
});

describe('createTransportAdapter', () => {
  it('type "mcp" returns McpTransportAdapter', () => {
    const adapter = createTransportAdapter({ type: 'mcp', endpoint: 'http://localhost:3000' });
    expect(adapter).toBeInstanceOf(McpTransportAdapter);
  });

  it('type "rest" returns RestTransportAdapter', () => {
    const adapter = createTransportAdapter({ type: 'rest', baseUrl: 'http://localhost:8080' });
    expect(adapter).toBeInstanceOf(RestTransportAdapter);
  });

  it('type "ws" returns WebSocketTransportAdapter', () => {
    const adapter = createTransportAdapter({ type: 'ws', url: 'ws://localhost:3001' });
    expect(adapter).toBeInstanceOf(WebSocketTransportAdapter);
  });

  it('unknown type returns StubTransportAdapter', () => {
    const adapter = createTransportAdapter({ type: 'unknown-protocol' });
    expect(adapter).toBeInstanceOf(StubTransportAdapter);
  });

  it('unknown type stub uses the type as protocol name', () => {
    const adapter = createTransportAdapter({ type: 'mqtt' });
    expect(adapter.protocol()).toBe('mqtt');
  });
});

describe('McpTransportAdapter', () => {
  it('protocol() returns "mcp"', () => {
    const adapter = new McpTransportAdapter('http://localhost:3000');
    expect(adapter.protocol()).toBe('mcp');
  });

  it('constructor does not throw', () => {
    expect(() => new McpTransportAdapter('http://localhost:3000')).not.toThrow();
  });

  it('constructor accepts authToken option', () => {
    expect(() => new McpTransportAdapter('http://localhost:3000', { authToken: 'tok123' })).not.toThrow();
  });

  it('constructor accepts custom headers', () => {
    expect(() => new McpTransportAdapter('http://localhost:3000', {
      headers: { 'X-Custom': 'value' },
    })).not.toThrow();
  });
});

describe('RestTransportAdapter', () => {
  it('protocol() returns "rest"', () => {
    const adapter = new RestTransportAdapter('http://localhost:8080');
    expect(adapter.protocol()).toBe('rest');
  });

  it('constructor does not throw', () => {
    expect(() => new RestTransportAdapter('http://localhost:8080')).not.toThrow();
  });

  it('constructor accepts authToken option', () => {
    expect(() => new RestTransportAdapter('http://localhost:8080', { authToken: 'bearer-tok' })).not.toThrow();
  });
});

describe('WebSocketTransportAdapter', () => {
  it('protocol() returns "ws"', () => {
    const adapter = new WebSocketTransportAdapter('ws://localhost:3001');
    expect(adapter.protocol()).toBe('ws');
  });

  it('constructor does not throw', () => {
    expect(() => new WebSocketTransportAdapter('ws://localhost:3001')).not.toThrow();
  });
});

describe('TransportAdapter interface conformance', () => {
  it('all adapters implement the TransportAdapter interface', () => {
    const adapters: TransportAdapter[] = [
      new StubTransportAdapter(),
      new McpTransportAdapter('http://localhost:3000'),
      new RestTransportAdapter('http://localhost:8080'),
      new WebSocketTransportAdapter('ws://localhost:3001'),
    ];

    for (const adapter of adapters) {
      expect(typeof adapter.send).toBe('function');
      expect(typeof adapter.healthCheck).toBe('function');
      expect(typeof adapter.protocol).toBe('function');
    }
  });
});
