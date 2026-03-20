import { describe, it, expect } from 'vitest';
import {
  StubTransportAdapter,
  McpTransportAdapter,
  RestTransportAdapter,
  WebSocketTransportAdapter,
  createTransportAdapter,
} from '../src/transport.js';
import { ClaudeCodeBridgeAdapter } from '../src/bridge-claude-code.js';
import { CodexBridgeAdapter } from '../src/bridge-codex.js';
import { HttpGenericBridgeAdapter } from '../src/bridge-http-generic.js';
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

  it('unsupported transport type returns StubTransportAdapter', () => {
    const adapter = createTransportAdapter({ type: 'quic', addr: '127.0.0.1:4433' });
    expect(adapter).toBeInstanceOf(StubTransportAdapter);
  });

  it('mqtt type returns StubTransportAdapter with correct protocol', () => {
    const adapter = createTransportAdapter({ type: 'mqtt', broker: 'mqtt://localhost', topic: 'test' });
    expect(adapter).toBeInstanceOf(StubTransportAdapter);
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
      new ClaudeCodeBridgeAdapter({}),
      new CodexBridgeAdapter({}),
      new HttpGenericBridgeAdapter({}),
    ];

    for (const adapter of adapters) {
      expect(typeof adapter.send).toBe('function');
      expect(typeof adapter.healthCheck).toBe('function');
      expect(typeof adapter.protocol).toBe('function');
    }
  });
});

describe('ClaudeCodeBridgeAdapter', () => {
  it('protocol() returns "bridge:claude-code"', () => {
    const adapter = new ClaudeCodeBridgeAdapter({});
    expect(adapter.protocol()).toBe('bridge:claude-code');
  });

  it('constructor accepts command override', () => {
    const adapter = new ClaudeCodeBridgeAdapter({ command: '/usr/local/bin/claude' });
    expect(adapter.protocol()).toBe('bridge:claude-code');
  });

  it('constructor accepts model option', () => {
    const adapter = new ClaudeCodeBridgeAdapter({ model: 'claude-sonnet-4-20250514' });
    expect(adapter.protocol()).toBe('bridge:claude-code');
  });
});

describe('CodexBridgeAdapter', () => {
  it('protocol() returns "bridge:codex"', () => {
    const adapter = new CodexBridgeAdapter({});
    expect(adapter.protocol()).toBe('bridge:codex');
  });

  it('constructor accepts endpoint and model', () => {
    const adapter = new CodexBridgeAdapter({
      endpoint: 'https://api.openai.com/v1/responses',
      model: 'codex-mini-latest',
    });
    expect(adapter.protocol()).toBe('bridge:codex');
  });
});

describe('HttpGenericBridgeAdapter', () => {
  it('protocol() returns "bridge:http-generic"', () => {
    const adapter = new HttpGenericBridgeAdapter({});
    expect(adapter.protocol()).toBe('bridge:http-generic');
  });

  it('constructor accepts endpoint, model, and authEnvVar', () => {
    const adapter = new HttpGenericBridgeAdapter({
      endpoint: 'http://localhost:11434',
      model: 'llama-3',
      authEnvVar: 'LLM_API_KEY',
    });
    expect(adapter.protocol()).toBe('bridge:http-generic');
  });
});

describe('createTransportAdapter bridge dispatch', () => {
  it('type "bridge" runtime "claude-code" returns ClaudeCodeBridgeAdapter', () => {
    const adapter = createTransportAdapter({
      type: 'bridge',
      runtime: 'claude-code',
      config: { command: 'claude' },
    });
    expect(adapter).toBeInstanceOf(ClaudeCodeBridgeAdapter);
    expect(adapter.protocol()).toBe('bridge:claude-code');
  });

  it('type "bridge" runtime "codex" returns CodexBridgeAdapter', () => {
    const adapter = createTransportAdapter({
      type: 'bridge',
      runtime: 'codex',
      config: { endpoint: 'https://api.openai.com/v1/responses' },
    });
    expect(adapter).toBeInstanceOf(CodexBridgeAdapter);
    expect(adapter.protocol()).toBe('bridge:codex');
  });

  it('type "bridge" runtime "http-generic" returns HttpGenericBridgeAdapter', () => {
    const adapter = createTransportAdapter({
      type: 'bridge',
      runtime: 'http-generic',
      config: { endpoint: 'http://localhost:8080' },
    });
    expect(adapter).toBeInstanceOf(HttpGenericBridgeAdapter);
    expect(adapter.protocol()).toBe('bridge:http-generic');
  });

  it('type "bridge" runtime "cursor" returns StubTransportAdapter', () => {
    const adapter = createTransportAdapter({
      type: 'bridge',
      runtime: 'cursor',
      config: {},
    });
    expect(adapter).toBeInstanceOf(StubTransportAdapter);
    expect(adapter.protocol()).toBe('bridge:cursor');
  });

  it('type "bridge" runtime "opencode" returns StubTransportAdapter', () => {
    const adapter = createTransportAdapter({
      type: 'bridge',
      runtime: 'opencode',
      config: {},
    });
    expect(adapter).toBeInstanceOf(StubTransportAdapter);
    expect(adapter.protocol()).toBe('bridge:opencode');
  });
});

describe('bridge templates', () => {
  it('claude-code-bridge template exists', async () => {
    const { getTemplate } = await import('../src/templates.js');
    const t = getTemplate('claude-code-bridge');
    expect(t).toBeDefined();
    expect(t!.name).toBe('Claude Code Bridge');
    expect(t!.origin.type).toBe('external');
  });

  it('codex-bridge template exists', async () => {
    const { getTemplate } = await import('../src/templates.js');
    const t = getTemplate('codex-bridge');
    expect(t).toBeDefined();
    expect(t!.name).toBe('Codex Bridge');
  });

  it('cursor-bridge template exists', async () => {
    const { getTemplate } = await import('../src/templates.js');
    const t = getTemplate('cursor-bridge');
    expect(t).toBeDefined();
    expect(t!.name).toBe('Cursor Bridge');
  });

  it('generic-llm-bridge template exists', async () => {
    const { getTemplate } = await import('../src/templates.js');
    const t = getTemplate('generic-llm-bridge');
    expect(t).toBeDefined();
    expect(t!.name).toBe('Generic LLM Bridge');
  });
});
