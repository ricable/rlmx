import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AgentType } from '@aix/shared';
import { AgentSpawner } from '../src/spawn.js';
import {
  AgentNotFoundError,
  AlreadyTerminatedError,
  MaxInstancesExceededError,
  SpawnNotAllowedError,
} from '../src/errors.js';
import type { AgentConfig } from '../src/types.js';

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function workerConfig(overrides: Partial<AgentConfig> = {}): AgentConfig {
  return {
    agentType: AgentType.Worker,
    name: 'test-worker',
    task: 'test task',
    ...overrides,
  };
}

function coordinatorConfig(overrides: Partial<AgentConfig> = {}): AgentConfig {
  return {
    agentType: AgentType.Coordinator,
    name: 'coordinator',
    ...overrides,
  };
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe('AgentSpawner', () => {
  let spawner: AgentSpawner;

  beforeEach(() => {
    spawner = new AgentSpawner();
  });

  it('spawns an agent and returns an id', () => {
    const id = spawner.spawn(workerConfig());
    expect(typeof id).toBe('string');
    expect(id.length).toBeGreaterThan(0);
  });

  it('agent info is retrievable after spawn', () => {
    const id = spawner.spawn(workerConfig());
    const info = spawner.get(id);
    expect(info).toBeDefined();
    expect(info!.agentType).toBe(AgentType.Worker);
    expect(info!.status).toBe('Running');
    expect(info!.name).toBe('test-worker');
    expect(info!.task).toBe('test task');
  });

  it('uses preferred zone when none specified', () => {
    const id = spawner.spawn(workerConfig({ zone: undefined }));
    expect(spawner.get(id)!.zone).toBe('Multi'); // Worker preferred zone
  });

  it('uses custom zone when specified', () => {
    const id = spawner.spawn(workerConfig({ zone: 'X' }));
    expect(spawner.get(id)!.zone).toBe('X');
  });

  it('auto-generates name when not specified', () => {
    const id = spawner.spawn(workerConfig({ name: undefined }));
    const info = spawner.get(id)!;
    expect(info.name).toMatch(/^Worker-/);
  });

  it('sets correct model tier', () => {
    const workerId = spawner.spawn(workerConfig());
    expect(spawner.get(workerId)!.modelTier).toBe('Small');

    const coordId = spawner.spawn(coordinatorConfig());
    expect(spawner.get(coordId)!.modelTier).toBe('Medium');
  });

  it('sets spawnedAt to a recent date', () => {
    const before = Date.now();
    const id = spawner.spawn(workerConfig());
    const after = Date.now();
    const spawnedAt = spawner.get(id)!.spawnedAt.getTime();
    expect(spawnedAt).toBeGreaterThanOrEqual(before);
    expect(spawnedAt).toBeLessThanOrEqual(after);
  });

  it('initializes metrics to zero', () => {
    const id = spawner.spawn(workerConfig());
    const metrics = spawner.get(id)!.metrics;
    expect(metrics.tasksCompleted).toBe(0);
    expect(metrics.tasksFailed).toBe(0);
    expect(metrics.uptimeSecs).toBe(0);
    expect(metrics.memoryUsageMb).toBe(0);
    expect(metrics.cpuUsagePct).toBe(0);
  });
});

describe('AgentSpawner — terminate', () => {
  let spawner: AgentSpawner;

  beforeEach(() => {
    spawner = new AgentSpawner();
  });

  it('terminates an agent', () => {
    const id = spawner.spawn(workerConfig());
    spawner.terminate(id);
    expect(spawner.get(id)!.status).toBe('Terminated');
  });

  it('throws AgentNotFoundError for unknown id', () => {
    expect(() => spawner.terminate('nonexistent')).toThrow(AgentNotFoundError);
  });

  it('throws AlreadyTerminatedError on double terminate', () => {
    const id = spawner.spawn(workerConfig());
    spawner.terminate(id);
    expect(() => spawner.terminate(id)).toThrow(AlreadyTerminatedError);
  });
});

describe('AgentSpawner — max instances', () => {
  let spawner: AgentSpawner;

  beforeEach(() => {
    spawner = new AgentSpawner();
  });

  it('enforces max instances (Coordinator limit = 1)', () => {
    spawner.spawn(coordinatorConfig());
    expect(() => spawner.spawn(coordinatorConfig())).toThrow(MaxInstancesExceededError);
  });

  it('terminated agents do not count toward limit', () => {
    const id = spawner.spawn(coordinatorConfig());
    spawner.terminate(id);
    // Should be able to spawn another
    const id2 = spawner.spawn(coordinatorConfig());
    expect(spawner.get(id2)!.status).toBe('Running');
  });
});

describe('AgentSpawner — spawn hierarchy', () => {
  let spawner: AgentSpawner;

  beforeEach(() => {
    spawner = new AgentSpawner();
  });

  it('Coordinator can spawn Worker', () => {
    const coordId = spawner.spawn(coordinatorConfig());
    const workerId = spawner.spawn(workerConfig(), coordId);
    expect(spawner.get(workerId)).toBeDefined();
  });

  it('parent-child relationship is recorded', () => {
    const coordId = spawner.spawn(coordinatorConfig());
    const workerId = spawner.spawn(workerConfig(), coordId);

    const coord = spawner.get(coordId)!;
    expect(coord.children).toContain(workerId);

    const worker = spawner.get(workerId)!;
    expect(worker.parentId).toBe(coordId);
  });

  it('Worker cannot spawn another Worker', () => {
    const workerId = spawner.spawn(workerConfig());
    expect(() => spawner.spawn(workerConfig(), workerId)).toThrow(SpawnNotAllowedError);
  });

  it('throws AgentNotFoundError for unknown parent', () => {
    expect(() => spawner.spawn(workerConfig(), 'nonexistent')).toThrow(AgentNotFoundError);
  });
});

describe('AgentSpawner — listing', () => {
  let spawner: AgentSpawner;

  beforeEach(() => {
    spawner = new AgentSpawner();
  });

  it('list() returns all agents', () => {
    spawner.spawn(workerConfig());
    spawner.spawn(coordinatorConfig());
    expect(spawner.list()).toHaveLength(2);
  });

  it('listByType filters by type', () => {
    spawner.spawn(workerConfig());
    spawner.spawn(workerConfig({ name: 'worker-2' }));
    spawner.spawn(coordinatorConfig());

    expect(spawner.listByType(AgentType.Worker)).toHaveLength(2);
    expect(spawner.listByType(AgentType.Coordinator)).toHaveLength(1);
    expect(spawner.listByType(AgentType.Monitor)).toHaveLength(0);
  });

  it('countByType excludes terminated agents', () => {
    const id = spawner.spawn(workerConfig());
    spawner.spawn(workerConfig({ name: 'worker-2' }));
    expect(spawner.countByType(AgentType.Worker)).toBe(2);
    spawner.terminate(id);
    expect(spawner.countByType(AgentType.Worker)).toBe(1);
  });

  it('get returns undefined for unknown id', () => {
    expect(spawner.get('nonexistent')).toBeUndefined();
  });
});

describe('AgentSpawner — hasNativeBridge', () => {
  it('reports native bridge availability', () => {
    const spawner = new AgentSpawner();
    // In test environment, NAPI bridge is not available
    expect(typeof spawner.hasNativeBridge).toBe('boolean');
  });
});
