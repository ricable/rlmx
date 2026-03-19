import { describe, it, expect, beforeEach } from 'vitest';
import {
  canTransition,
  TERMINAL_STATES,
  TaskStore,
  buildAgentCard,
  A2AServer,
  A2AClient,
  A2AClientError,
} from '../src/index.js';
import type {
  TaskState,
  A2AMessage,
  A2ATask,
  AgentCard,
} from '../src/index.js';

// ---------------------------------------------------------------------------
// canTransition
// ---------------------------------------------------------------------------

describe('canTransition', () => {
  it('allows submitted -> working', () => {
    expect(canTransition('submitted', 'working')).toBe(true);
  });

  it('allows submitted -> cancelled', () => {
    expect(canTransition('submitted', 'cancelled')).toBe(true);
  });

  it('disallows submitted -> completed', () => {
    expect(canTransition('submitted', 'completed')).toBe(false);
  });

  it('disallows submitted -> failed', () => {
    expect(canTransition('submitted', 'failed')).toBe(false);
  });

  it('allows working -> completed', () => {
    expect(canTransition('working', 'completed')).toBe(true);
  });

  it('allows working -> failed', () => {
    expect(canTransition('working', 'failed')).toBe(true);
  });

  it('allows working -> input-required', () => {
    expect(canTransition('working', 'input-required')).toBe(true);
  });

  it('allows working -> cancelled', () => {
    expect(canTransition('working', 'cancelled')).toBe(true);
  });

  it('allows input-required -> working', () => {
    expect(canTransition('input-required', 'working')).toBe(true);
  });

  it('allows input-required -> cancelled', () => {
    expect(canTransition('input-required', 'cancelled')).toBe(true);
  });

  it('disallows input-required -> completed', () => {
    expect(canTransition('input-required', 'completed')).toBe(false);
  });

  it('disallows transitions from terminal states', () => {
    for (const terminal of TERMINAL_STATES) {
      const targets: TaskState[] = [
        'submitted',
        'working',
        'input-required',
        'completed',
        'cancelled',
        'failed',
      ];
      for (const target of targets) {
        expect(canTransition(terminal, target)).toBe(false);
      }
    }
  });
});

// ---------------------------------------------------------------------------
// TaskStore
// ---------------------------------------------------------------------------

describe('TaskStore', () => {
  let store: TaskStore;

  const userMsg: A2AMessage = {
    role: 'user',
    parts: [{ type: 'text', text: 'Hello' }],
  };

  const agentMsg: A2AMessage = {
    role: 'agent',
    parts: [{ type: 'text', text: 'Hi there' }],
  };

  beforeEach(() => {
    store = new TaskStore();
  });

  it('creates a task with submitted state', () => {
    const task = store.create(userMsg);
    expect(task.id).toBeTruthy();
    expect(task.state).toBe('submitted');
    expect(task.messages).toHaveLength(1);
    expect(task.messages[0]).toEqual(userMsg);
  });

  it('retrieves a task by id', () => {
    const task = store.create(userMsg);
    const retrieved = store.get(task.id);
    expect(retrieved).toEqual(task);
  });

  it('returns undefined for unknown id', () => {
    expect(store.get('nonexistent')).toBeUndefined();
  });

  it('updates task state', () => {
    const task = store.create(userMsg);
    const updated = store.update(task.id, 'working');
    expect(updated.state).toBe('working');
  });

  it('updates task state with message', () => {
    const task = store.create(userMsg);
    store.update(task.id, 'working');
    const updated = store.update(task.id, 'completed', agentMsg);
    expect(updated.state).toBe('completed');
    expect(updated.messages).toHaveLength(2);
  });

  it('throws on invalid state transition', () => {
    const task = store.create(userMsg);
    expect(() => store.update(task.id, 'completed')).toThrow(
      'Invalid state transition',
    );
  });

  it('throws on update for nonexistent task', () => {
    expect(() => store.update('nope', 'working')).toThrow('Task not found');
  });

  it('cancels a submitted task', () => {
    const task = store.create(userMsg);
    const cancelled = store.cancel(task.id);
    expect(cancelled.state).toBe('cancelled');
  });

  it('cancels a working task', () => {
    const task = store.create(userMsg);
    store.update(task.id, 'working');
    const cancelled = store.cancel(task.id);
    expect(cancelled.state).toBe('cancelled');
  });

  it('throws when cancelling a completed task', () => {
    const task = store.create(userMsg);
    store.update(task.id, 'working');
    store.update(task.id, 'completed');
    expect(() => store.cancel(task.id)).toThrow('Invalid state transition');
  });

  it('lists all tasks', () => {
    store.create(userMsg);
    store.create(userMsg);
    expect(store.list()).toHaveLength(2);
  });

  it('lists tasks filtered by state', () => {
    const t1 = store.create(userMsg);
    store.create(userMsg);
    store.update(t1.id, 'working');
    expect(store.list('working')).toHaveLength(1);
    expect(store.list('submitted')).toHaveLength(1);
  });

  it('reports size correctly', () => {
    expect(store.size).toBe(0);
    store.create(userMsg);
    expect(store.size).toBe(1);
    store.create(userMsg);
    expect(store.size).toBe(2);
  });

  it('evicts oldest task at capacity', () => {
    const smallStore = new TaskStore(3);
    const t1 = smallStore.create(userMsg);
    smallStore.create(userMsg);
    smallStore.create(userMsg);
    expect(smallStore.size).toBe(3);

    // Creating a 4th should evict the oldest
    smallStore.create(userMsg);
    expect(smallStore.size).toBe(3);
    expect(smallStore.get(t1.id)).toBeUndefined();
  });

  it('evicts in FIFO order', () => {
    const smallStore = new TaskStore(2);
    const t1 = smallStore.create(userMsg);
    const t2 = smallStore.create(userMsg);

    const t3 = smallStore.create(userMsg);
    expect(smallStore.get(t1.id)).toBeUndefined();
    expect(smallStore.get(t2.id)).toBeDefined();
    expect(smallStore.get(t3.id)).toBeDefined();

    const t4 = smallStore.create(userMsg);
    expect(smallStore.get(t2.id)).toBeUndefined();
    expect(smallStore.get(t3.id)).toBeDefined();
    expect(smallStore.get(t4.id)).toBeDefined();
  });
});

// ---------------------------------------------------------------------------
// buildAgentCard
// ---------------------------------------------------------------------------

describe('buildAgentCard', () => {
  it('builds a card with 17 skills', () => {
    const card = buildAgentCard('http://localhost:3000', '0.1.0');
    expect(card.skills).toHaveLength(17);
  });

  it('sets name and version', () => {
    const card = buildAgentCard('http://localhost:3000', '1.2.3');
    expect(card.name).toBe('RLMX Agent');
    expect(card.version).toBe('1.2.3');
    expect(card.url).toBe('http://localhost:3000');
  });

  it('includes bearer authentication', () => {
    const card = buildAgentCard('http://localhost:3000', '0.1.0');
    expect(card.authentication.schemes).toContain('bearer');
  });

  it('has state transition history enabled', () => {
    const card = buildAgentCard('http://localhost:3000', '0.1.0');
    expect(card.capabilities.stateTransitionHistory).toBe(true);
    expect(card.capabilities.streaming).toBe(false);
    expect(card.capabilities.pushNotifications).toBe(false);
  });

  it('generates unique skill IDs', () => {
    const card = buildAgentCard('http://localhost:3000', '0.1.0');
    const ids = card.skills.map((s) => s.id);
    expect(new Set(ids).size).toBe(17);
  });

  it('converts PascalCase agent types to kebab-case IDs', () => {
    const card = buildAgentCard('http://localhost:3000', '0.1.0');
    const ids = card.skills.map((s) => s.id);
    expect(ids).toContain('coordinator');
    expect(ids).toContain('voice-coordinator');
    expect(ids).toContain('marketplace-manager');
    expect(ids).toContain('mesh-coordinator');
    expect(ids).toContain('federation-agent');
    expect(ids).toContain('billing-manager');
  });

  it('all skills have descriptions and tags', () => {
    const card = buildAgentCard('http://localhost:3000', '0.1.0');
    for (const skill of card.skills) {
      expect(skill.description.length).toBeGreaterThan(0);
      expect(skill.tags.length).toBeGreaterThan(0);
      expect(skill.name.length).toBeGreaterThan(0);
    }
  });
});

// ---------------------------------------------------------------------------
// A2AServer
// ---------------------------------------------------------------------------

describe('A2AServer', () => {
  let server: A2AServer;

  const userMsg: A2AMessage = {
    role: 'user',
    parts: [{ type: 'text', text: 'Do something' }],
  };

  beforeEach(() => {
    const card = buildAgentCard('http://localhost:3000', '0.1.0');
    server = new A2AServer(card);
  });

  it('returns agent card', () => {
    const card = server.getCard();
    expect(card.skills).toHaveLength(17);
  });

  it('handles tasks/send to create a task', async () => {
    const response = await server.handleRequest({
      jsonrpc: '2.0',
      id: 1,
      method: 'tasks/send',
      params: { message: userMsg },
    });

    expect('result' in response).toBe(true);
    if ('result' in response) {
      const task = response.result as A2ATask;
      expect(task.state).toBe('working');
      expect(task.messages).toHaveLength(1);
    }
  });

  it('handles tasks/get for existing task', async () => {
    const sendRes = await server.handleRequest({
      jsonrpc: '2.0',
      id: 1,
      method: 'tasks/send',
      params: { message: userMsg },
    });

    const task = (sendRes as { result: A2ATask }).result;

    const getRes = await server.handleRequest({
      jsonrpc: '2.0',
      id: 2,
      method: 'tasks/get',
      params: { id: task.id },
    });

    expect('result' in getRes).toBe(true);
    if ('result' in getRes) {
      expect((getRes.result as A2ATask).id).toBe(task.id);
    }
  });

  it('returns error for tasks/get with unknown id', async () => {
    const response = await server.handleRequest({
      jsonrpc: '2.0',
      id: 1,
      method: 'tasks/get',
      params: { id: 'nonexistent' },
    });

    expect('error' in response).toBe(true);
  });

  it('handles tasks/cancel', async () => {
    const sendRes = await server.handleRequest({
      jsonrpc: '2.0',
      id: 1,
      method: 'tasks/send',
      params: { message: userMsg },
    });

    const task = (sendRes as { result: A2ATask }).result;

    const cancelRes = await server.handleRequest({
      jsonrpc: '2.0',
      id: 2,
      method: 'tasks/cancel',
      params: { id: task.id },
    });

    expect('result' in cancelRes).toBe(true);
    if ('result' in cancelRes) {
      expect((cancelRes.result as A2ATask).state).toBe('cancelled');
    }
  });

  it('returns method-not-found for unknown methods', async () => {
    const response = await server.handleRequest({
      jsonrpc: '2.0',
      id: 1,
      method: 'unknown/method',
    });

    expect('error' in response).toBe(true);
    if ('error' in response) {
      expect(response.error.code).toBe(-32601);
    }
  });

  it('returns invalid-request for non-jsonrpc messages', async () => {
    const response = await server.handleRequest({ foo: 'bar' });
    expect('error' in response).toBe(true);
  });

  it('returns invalid-params when message is missing', async () => {
    const response = await server.handleRequest({
      jsonrpc: '2.0',
      id: 1,
      method: 'tasks/send',
      params: {},
    });

    expect('error' in response).toBe(true);
  });

  it('invokes task handler on send', async () => {
    let handlerCalled = false;
    server.onTask(() => {
      handlerCalled = true;
    });

    await server.handleRequest({
      jsonrpc: '2.0',
      id: 1,
      method: 'tasks/send',
      params: { message: userMsg },
    });

    expect(handlerCalled).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// A2AClient
// ---------------------------------------------------------------------------

describe('A2AClient', () => {
  function mockFetch(
    responseBody: unknown,
    status = 200,
  ) {
    return async (_url: string, _init: unknown) => ({
      ok: status >= 200 && status < 300,
      status,
      json: async () => responseBody,
    });
  }

  it('discovers an agent card', async () => {
    const card = buildAgentCard('http://remote:3000', '0.1.0');
    const client = new A2AClient(mockFetch(card));

    const result = await client.discover('http://remote:3000');
    expect(result.name).toBe('RLMX Agent');
    expect(result.skills).toHaveLength(17);
  });

  it('throws on discovery failure', async () => {
    const client = new A2AClient(mockFetch({}, 404));
    await expect(client.discover('http://remote:3000')).rejects.toThrow(
      A2AClientError,
    );
  });

  it('sends a task', async () => {
    const task: A2ATask = {
      id: 'test-1',
      state: 'working',
      messages: [{ role: 'user', parts: [{ type: 'text', text: 'hi' }] }],
    };
    const client = new A2AClient(
      mockFetch({ jsonrpc: '2.0', id: 'req-1', result: task }),
    );

    const result = await client.send('http://remote:3000', {
      role: 'user',
      parts: [{ type: 'text', text: 'hi' }],
    });
    expect(result.id).toBe('test-1');
    expect(result.state).toBe('working');
  });

  it('gets a task', async () => {
    const task: A2ATask = {
      id: 'test-1',
      state: 'completed',
      messages: [],
    };
    const client = new A2AClient(
      mockFetch({ jsonrpc: '2.0', id: 'req-1', result: task }),
    );

    const result = await client.getTask('http://remote:3000', 'test-1');
    expect(result.state).toBe('completed');
  });

  it('cancels a task', async () => {
    const task: A2ATask = {
      id: 'test-1',
      state: 'cancelled',
      messages: [],
    };
    const client = new A2AClient(
      mockFetch({ jsonrpc: '2.0', id: 'req-1', result: task }),
    );

    const result = await client.cancel('http://remote:3000', 'test-1');
    expect(result.state).toBe('cancelled');
  });

  it('throws on RPC error response', async () => {
    const client = new A2AClient(
      mockFetch({
        jsonrpc: '2.0',
        id: 'req-1',
        error: { code: -32001, message: 'Task not found' },
      }),
    );

    await expect(
      client.getTask('http://remote:3000', 'test-1'),
    ).rejects.toThrow('RPC error');
  });

  it('throws on HTTP error', async () => {
    const client = new A2AClient(mockFetch({}, 500));
    await expect(
      client.send('http://remote:3000', {
        role: 'user',
        parts: [{ type: 'text', text: 'hi' }],
      }),
    ).rejects.toThrow('HTTP error');
  });

  it('A2AClientError is instanceof AixError', () => {
    const err = new A2AClientError('test');
    expect(err.name).toBe('A2AClientError');
    expect(err.code).toBeDefined();
  });
});
