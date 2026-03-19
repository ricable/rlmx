import { describe, it, expect, vi } from 'vitest';
import {
  DomainEventBus,
  createEventBus,
  DOMAIN_EVENT_TYPES,
  type DomainEvent,
  type SyscallDispatchedEvent,
  type AgentSpawnedEvent,
  type FederationCycleCompletedEvent,
} from '../src/events.js';

describe('DOMAIN_EVENT_TYPES', () => {
  it('has exactly 10 event types', () => {
    expect(DOMAIN_EVENT_TYPES).toHaveLength(10);
  });

  it('contains all expected types', () => {
    const expected = [
      'SyscallDispatched',
      'AgentSpawned',
      'QueryRouted',
      'ExperimentCompleted',
      'NodeMembershipChanged',
      'StateMutated',
      'VoiceSessionStarted',
      'IntentsDecomposed',
      'MeshDeviceJoined',
      'FederationCycleCompleted',
    ];
    expect([...DOMAIN_EVENT_TYPES]).toEqual(expected);
  });
});

describe('DomainEventBus', () => {
  it('can be created with createEventBus helper', () => {
    const bus = createEventBus();
    expect(bus).toBeInstanceOf(DomainEventBus);
  });

  it('emits and receives typed events', () => {
    const bus = createEventBus();
    const handler = vi.fn();

    bus.on('SyscallDispatched', handler);

    const event: SyscallDispatchedEvent = {
      type: 'SyscallDispatched',
      syscallType: 'VecInsert',
      processId: 42,
      timestamp: new Date().toISOString(),
      success: true,
    };

    const hadListeners = bus.emit(event);
    expect(hadListeners).toBe(true);
    expect(handler).toHaveBeenCalledOnce();
    expect(handler).toHaveBeenCalledWith(event);
  });

  it('does not fire handler for unrelated event types', () => {
    const bus = createEventBus();
    const handler = vi.fn();

    bus.on('AgentSpawned', handler);

    const event: SyscallDispatchedEvent = {
      type: 'SyscallDispatched',
      syscallType: 'VecSearch',
      processId: 1,
      timestamp: new Date().toISOString(),
      success: true,
    };

    bus.emit(event);
    expect(handler).not.toHaveBeenCalled();
  });

  it('supports once() for single-fire listeners', () => {
    const bus = createEventBus();
    const handler = vi.fn();

    bus.once('StateMutated', handler);

    const event: DomainEvent = {
      type: 'StateMutated',
      witnessId: 'abc-123',
      success: true,
      timestamp: new Date().toISOString(),
    };

    bus.emit(event);
    bus.emit(event);

    expect(handler).toHaveBeenCalledOnce();
  });

  it('supports off() to unsubscribe', () => {
    const bus = createEventBus();
    const handler = vi.fn();

    bus.on('QueryRouted', handler);
    bus.off('QueryRouted', handler);

    bus.emit({
      type: 'QueryRouted',
      queryHash: 12345,
      strategy: 'Rlm',
      confidence: 0.9,
      timestamp: new Date().toISOString(),
    });

    expect(handler).not.toHaveBeenCalled();
  });

  it('supports onAny() to receive all event types', () => {
    const bus = createEventBus();
    const handler = vi.fn();

    bus.onAny(handler);

    const events: DomainEvent[] = [
      {
        type: 'SyscallDispatched',
        syscallType: 'GraphQuery',
        processId: 1,
        timestamp: new Date().toISOString(),
        success: true,
      },
      {
        type: 'AgentSpawned',
        agentId: 'agent-1',
        agentType: 'Worker',
        parentId: null,
        timestamp: new Date().toISOString(),
      },
      {
        type: 'FederationCycleCompleted',
        cycleId: 'cycle-1',
        contributors: 1500,
        patternsAggregated: 42000,
        timestamp: new Date().toISOString(),
      },
    ];

    for (const event of events) {
      bus.emit(event);
    }

    expect(handler).toHaveBeenCalledTimes(3);
  });

  it('returns false when no listeners are registered', () => {
    const bus = createEventBus();

    const result = bus.emit({
      type: 'StateMutated',
      witnessId: 'w-1',
      success: true,
      timestamp: new Date().toISOString(),
    });

    expect(result).toBe(false);
  });

  it('reports correct listenerCount', () => {
    const bus = createEventBus();

    expect(bus.listenerCount('AgentSpawned')).toBe(0);

    const h1 = vi.fn();
    const h2 = vi.fn();
    bus.on('AgentSpawned', h1);
    bus.on('AgentSpawned', h2);

    expect(bus.listenerCount('AgentSpawned')).toBe(2);
  });

  it('removeAllListeners clears specific type', () => {
    const bus = createEventBus();
    bus.on('AgentSpawned', vi.fn());
    bus.on('AgentSpawned', vi.fn());
    bus.on('QueryRouted', vi.fn());

    bus.removeAllListeners('AgentSpawned');

    expect(bus.listenerCount('AgentSpawned')).toBe(0);
    expect(bus.listenerCount('QueryRouted')).toBe(1);
  });

  it('removeAllListeners with no arg clears everything', () => {
    const bus = createEventBus();
    bus.on('AgentSpawned', vi.fn());
    bus.on('QueryRouted', vi.fn());
    bus.on('StateMutated', vi.fn());

    bus.removeAllListeners();

    expect(bus.listenerCount('AgentSpawned')).toBe(0);
    expect(bus.listenerCount('QueryRouted')).toBe(0);
    expect(bus.listenerCount('StateMutated')).toBe(0);
  });

  it('handles all 10 event types correctly', () => {
    const bus = createEventBus();
    const received: string[] = [];

    bus.onAny((event) => {
      received.push(event.type);
    });

    const now = new Date().toISOString();

    const allEvents: DomainEvent[] = [
      { type: 'SyscallDispatched', syscallType: 'VecInsert', processId: 1, timestamp: now, success: true },
      { type: 'AgentSpawned', agentId: 'a', agentType: 'Worker', parentId: null, timestamp: now },
      { type: 'QueryRouted', queryHash: 1, strategy: 'Auto', confidence: 1, timestamp: now },
      { type: 'ExperimentCompleted', experimentId: 'e', fitness: 0.9, generation: 1, timestamp: now },
      { type: 'NodeMembershipChanged', nodeId: 'n', joined: true, zone: 'A', timestamp: now },
      { type: 'StateMutated', witnessId: 'w', success: true, timestamp: now },
      { type: 'VoiceSessionStarted', sessionId: 's', mode: 'multimodal', timestamp: now },
      { type: 'IntentsDecomposed', sessionId: 's', intentCount: 3, domains: ['Finance'], timestamp: now },
      { type: 'MeshDeviceJoined', meshId: 'm', deviceId: 'd', zone: 'A-Desktop', timestamp: now },
      { type: 'FederationCycleCompleted', cycleId: 'c', contributors: 1500, patternsAggregated: 42000, timestamp: now },
    ];

    for (const event of allEvents) {
      bus.emit(event);
    }

    expect(received).toEqual(DOMAIN_EVENT_TYPES);
  });
});
