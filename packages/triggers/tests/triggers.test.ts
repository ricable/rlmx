import { describe, it, expect, beforeEach } from 'vitest';
import {
  TriggerRegistry,
  matchEvent,
  matchHttp,
  matchChannel,
  execute,
} from '../src/index.js';
import type { TriggerType, TriggerBinding } from '../src/index.js';

describe('TriggerRegistry', () => {
  let registry: TriggerRegistry;

  beforeEach(() => {
    registry = new TriggerRegistry();
  });

  // --- CRUD ---

  it('should create a binding and return an id', () => {
    const id = registry.bind({ type: 'event', domainEvent: 'StateMutated' }, 'handler');
    expect(id).toBeTruthy();
    expect(registry.get(id)).toBeDefined();
  });

  it('should unbind a binding', () => {
    const id = registry.bind({ type: 'event', domainEvent: 'Test' }, 'handler');
    expect(registry.unbind(id)).toBe(true);
    expect(registry.get(id)).toBeUndefined();
  });

  it('should return false when unbinding nonexistent', () => {
    expect(registry.unbind('nonexistent')).toBe(false);
  });

  it('should enable and disable a binding', () => {
    const id = registry.bind({ type: 'event', domainEvent: 'Test' }, 'handler');
    registry.disable(id);
    expect(registry.get(id)?.enabled).toBe(false);
    registry.enable(id);
    expect(registry.get(id)?.enabled).toBe(true);
  });

  it('should return false when enabling nonexistent', () => {
    expect(registry.enable('nonexistent')).toBe(false);
  });

  it('should list all bindings', () => {
    registry.bind({ type: 'event', domainEvent: 'A' }, 'a');
    registry.bind({ type: 'event', domainEvent: 'B' }, 'b');
    expect(registry.list()).toHaveLength(2);
  });

  it('should store transform', () => {
    const id = registry.bind(
      { type: 'http', path: '/hook', method: 'POST' },
      'handler',
      '.body.data',
    );
    expect(registry.get(id)?.transform).toBe('.body.data');
  });

  it('should set max depth', () => {
    const id = registry.bind({ type: 'event', domainEvent: 'Test' }, 'handler');
    registry.setMaxDepth(id, 3);
    expect(registry.get(id)?.maxDepth).toBe(3);
  });

  // --- Event matching ---

  it('should match event bindings', () => {
    registry.bind({ type: 'event', domainEvent: 'StateMutated' }, 'handler1');
    registry.bind({ type: 'event', domainEvent: 'AgentSpawned' }, 'handler2');
    const matches = registry.matchEvent('StateMutated');
    expect(matches).toHaveLength(1);
    expect(matches[0].targetFunction).toBe('handler1');
  });

  it('should skip disabled event bindings', () => {
    const id = registry.bind({ type: 'event', domainEvent: 'StateMutated' }, 'handler');
    registry.disable(id);
    expect(registry.matchEvent('StateMutated')).toHaveLength(0);
  });

  it('should match multiple event bindings', () => {
    registry.bind({ type: 'event', domainEvent: 'StateMutated' }, 'handler1');
    registry.bind({ type: 'event', domainEvent: 'StateMutated' }, 'handler2');
    expect(registry.matchEvent('StateMutated')).toHaveLength(2);
  });

  // --- HTTP matching ---

  it('should match HTTP bindings', () => {
    registry.bind({ type: 'http', path: '/webhooks/github', method: 'POST' }, 'github_handler');
    const matches = registry.matchHttp('/webhooks/github', 'POST');
    expect(matches).toHaveLength(1);
  });

  it('should match HTTP method case-insensitively', () => {
    registry.bind({ type: 'http', path: '/hook', method: 'POST' }, 'handler');
    expect(registry.matchHttp('/hook', 'post')).toHaveLength(1);
  });

  it('should not match wrong HTTP method', () => {
    registry.bind({ type: 'http', path: '/hook', method: 'POST' }, 'handler');
    expect(registry.matchHttp('/hook', 'GET')).toHaveLength(0);
  });

  it('should not match wrong HTTP path', () => {
    registry.bind({ type: 'http', path: '/hook', method: 'POST' }, 'handler');
    expect(registry.matchHttp('/other', 'POST')).toHaveLength(0);
  });

  // --- Channel matching ---

  it('should match channel bindings by adapter and content', () => {
    registry.bind({ type: 'channel', adapter: 'telegram', filter: 'help' }, 'help_handler');
    const matches = registry.matchChannel('telegram', 'I need help please');
    expect(matches).toHaveLength(1);
  });

  it('should not match wrong adapter', () => {
    registry.bind({ type: 'channel', adapter: 'telegram', filter: 'help' }, 'handler');
    expect(registry.matchChannel('discord', 'help me')).toHaveLength(0);
  });

  it('should not match wrong content', () => {
    registry.bind({ type: 'channel', adapter: 'telegram', filter: 'help' }, 'handler');
    expect(registry.matchChannel('telegram', 'goodbye')).toHaveLength(0);
  });

  it('should match empty filter against any content', () => {
    registry.bind({ type: 'channel', adapter: 'telegram', filter: '' }, 'catch_all');
    expect(registry.matchChannel('telegram', 'anything')).toHaveLength(1);
  });

  // --- Depth tracking ---

  it('should allow invocation initially', () => {
    expect(registry.checkDepth('my_fn')).toBe(true);
  });

  it('should track enter and exit depth', () => {
    registry.bind({ type: 'event', domainEvent: 'Test' }, 'my_fn');
    registry.enterDepth('my_fn');
    expect(registry.checkDepth('my_fn')).toBe(true);
    registry.exitDepth('my_fn');
    expect(registry.checkDepth('my_fn')).toBe(true);
  });

  it('should block when depth reaches max', () => {
    const id = registry.bind({ type: 'event', domainEvent: 'Test' }, 'my_fn');
    registry.setMaxDepth(id, 2);
    registry.enterDepth('my_fn');
    registry.enterDepth('my_fn');
    expect(registry.checkDepth('my_fn')).toBe(false);
  });

  it('should recover after exit', () => {
    const id = registry.bind({ type: 'event', domainEvent: 'Test' }, 'my_fn');
    registry.setMaxDepth(id, 2);
    registry.enterDepth('my_fn');
    registry.enterDepth('my_fn');
    expect(registry.checkDepth('my_fn')).toBe(false);
    registry.exitDepth('my_fn');
    expect(registry.checkDepth('my_fn')).toBe(true);
  });
});

describe('standalone matchers', () => {
  const bindings: TriggerBinding[] = [
    {
      id: 'b1',
      triggerType: { type: 'event', domainEvent: 'StateMutated' },
      targetFunction: 'handler1',
      transform: null,
      enabled: true,
      maxDepth: 5,
    },
    {
      id: 'b2',
      triggerType: { type: 'http', path: '/hook', method: 'POST' },
      targetFunction: 'handler2',
      transform: null,
      enabled: true,
      maxDepth: 5,
    },
    {
      id: 'b3',
      triggerType: { type: 'channel', adapter: 'telegram', filter: 'help' },
      targetFunction: 'handler3',
      transform: null,
      enabled: true,
      maxDepth: 5,
    },
    {
      id: 'b4',
      triggerType: { type: 'event', domainEvent: 'StateMutated' },
      targetFunction: 'handler4',
      transform: null,
      enabled: false,
      maxDepth: 5,
    },
  ];

  it('matchEvent returns matching enabled bindings', () => {
    expect(matchEvent(bindings, 'StateMutated')).toHaveLength(1);
  });

  it('matchHttp returns matching bindings', () => {
    expect(matchHttp(bindings, '/hook', 'POST')).toHaveLength(1);
  });

  it('matchChannel returns matching bindings', () => {
    expect(matchChannel(bindings, 'telegram', 'I need help')).toHaveLength(1);
  });
});

describe('executor', () => {
  it('should return success with target function', () => {
    const binding: TriggerBinding = {
      id: 'test-id',
      triggerType: { type: 'event', domainEvent: 'Test' },
      targetFunction: 'my_handler',
      transform: null,
      enabled: true,
      maxDepth: 5,
    };
    const result = execute(binding, { data: 'test' });
    expect(result.success).toBe(true);
    expect(result.targetFunction).toBe('my_handler');
    expect(result.input).toEqual({ data: 'test' });
  });

  it('should apply transform wrapper when transform is set', () => {
    const binding: TriggerBinding = {
      id: 'test-id',
      triggerType: { type: 'event', domainEvent: 'Test' },
      targetFunction: 'my_handler',
      transform: '.body.data',
      enabled: true,
      maxDepth: 5,
    };
    const result = execute(binding, { raw: 'value' });
    expect(result.success).toBe(true);
    expect(result.input).toEqual({
      _transform: '.body.data',
      _original: { raw: 'value' },
    });
  });

  it('should include binding id in result', () => {
    const binding: TriggerBinding = {
      id: 'binding-123',
      triggerType: { type: 'event', domainEvent: 'Test' },
      targetFunction: 'handler',
      transform: null,
      enabled: true,
      maxDepth: 5,
    };
    const result = execute(binding, null);
    expect(result.bindingId).toBe('binding-123');
  });
});
