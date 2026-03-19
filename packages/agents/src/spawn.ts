import { AgentType } from '@aix/shared';
import type { AgentConfig, AgentInfo } from './types.js';
import { defaultMetrics, modelTierFor, preferredZone, maxInstances } from './types.js';
import { PermissionRegistry } from './registry.js';
import {
  AgentNotFoundError,
  AlreadyTerminatedError,
  MaxInstancesExceededError,
  NapiBridgeUnavailableError,
  SpawnNotAllowedError,
} from './errors.js';

// ---------------------------------------------------------------------------
// Attempt to load the NAPI bridge at module level (graceful degradation)
// ---------------------------------------------------------------------------

type NapiAgentSpawnFn = (agentType: string, config: Record<string, unknown>) => unknown;

let napiAgentSpawn: NapiAgentSpawnFn | null = null;
let napiNativeAvailable = false;

try {
  // Dynamic import of the optional native bridge.
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  const core = require('@aix/core');
  if (typeof core?.napiAgentSpawn === 'function' && core?.isNativeAvailable === true) {
    napiAgentSpawn = core.napiAgentSpawn as NapiAgentSpawnFn;
    napiNativeAvailable = true;
  }
} catch {
  // NAPI bridge not available — pure-JS fallback will be used.
}

// ---------------------------------------------------------------------------
// ID generation fallback (no external dependency)
// ---------------------------------------------------------------------------

function generateId(): string {
  // crypto.randomUUID is available in Node 19+ and all modern browsers
  if (typeof globalThis.crypto?.randomUUID === 'function') {
    return globalThis.crypto.randomUUID();
  }
  // Fallback: pseudo-random hex string
  const bytes = new Uint8Array(16);
  if (typeof globalThis.crypto?.getRandomValues === 'function') {
    globalThis.crypto.getRandomValues(bytes);
  } else {
    for (let i = 0; i < 16; i++) {
      bytes[i] = Math.floor(Math.random() * 256);
    }
  }
  const hex = Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

// ---------------------------------------------------------------------------
// AgentSpawner — manages agent instances
// ---------------------------------------------------------------------------

/**
 * Manages agent lifecycle: spawn, terminate, list.
 *
 * Delegates to `@aix/core.napiAgentSpawn()` when the native bridge is
 * available. Falls back to a pure-JS in-memory implementation otherwise.
 */
export class AgentSpawner {
  private readonly agents = new Map<string, AgentInfo>();

  /** Whether the NAPI native bridge is available. */
  get hasNativeBridge(): boolean {
    return napiNativeAvailable;
  }

  /**
   * Spawn a new agent, validating max instances and spawn hierarchy.
   *
   * @param config - Agent configuration.
   * @param parentId - Optional parent agent id for hierarchy validation.
   * @returns The id of the newly spawned agent.
   */
  spawn(config: AgentConfig, parentId?: string): string {
    // Validate max instances
    const currentCount = this.countByType(config.agentType);
    const max = maxInstances(config.agentType);
    if (currentCount >= max) {
      throw new MaxInstancesExceededError(config.agentType, max);
    }

    // Validate spawn hierarchy if a parent is specified
    let resolvedParentId: string | undefined;
    if (parentId !== undefined) {
      const parent = this.agents.get(parentId);
      if (!parent) {
        throw new AgentNotFoundError(parentId);
      }
      if (!PermissionRegistry.canSpawn(parent.agentType, config.agentType)) {
        throw new SpawnNotAllowedError(parent.agentType, config.agentType);
      }
      resolvedParentId = parent.id;
    }

    // Try NAPI bridge first, fall back to local management
    let id: string;
    if (napiNativeAvailable && napiAgentSpawn) {
      try {
        const result = napiAgentSpawn(config.agentType, {
          name: config.name,
          task: config.task,
          zone: config.zone,
          maxRuntimeSecs: config.maxRuntimeSecs,
        });
        // NAPI returns a string id when native is available, otherwise an
        // unavailable-result object. Only accept string results.
        id = typeof result === 'string' ? result : generateId();
      } catch {
        // NAPI call failed — fall back to local id generation
        id = generateId();
      }
    } else {
      id = generateId();
    }

    const zone = config.zone ?? preferredZone(config.agentType);
    const name = config.name ?? `${config.agentType}-${id.slice(0, 8)}`;

    const info: AgentInfo = {
      id,
      agentType: config.agentType,
      name,
      status: 'Running',
      task: config.task,
      zone,
      spawnedAt: new Date(),
      parentId: resolvedParentId,
      children: [],
      modelTier: modelTierFor(config.agentType),
      metrics: defaultMetrics(),
    };

    this.agents.set(id, info);

    // Register as child of parent
    if (resolvedParentId !== undefined) {
      const parent = this.agents.get(resolvedParentId);
      if (parent) {
        parent.children.push(id);
      }
    }

    return id;
  }

  /**
   * Terminate an agent by id.
   */
  terminate(id: string): void {
    const agent = this.agents.get(id);
    if (!agent) {
      throw new AgentNotFoundError(id);
    }
    if (agent.status === 'Terminated') {
      throw new AlreadyTerminatedError();
    }
    agent.status = 'Terminated';
  }

  /** Get agent info by id. Returns undefined if not found. */
  get(id: string): AgentInfo | undefined {
    return this.agents.get(id);
  }

  /** List all agents. */
  list(): AgentInfo[] {
    return [...this.agents.values()];
  }

  /** List agents of a specific type. */
  listByType(agentType: AgentType): AgentInfo[] {
    return [...this.agents.values()].filter((a) => a.agentType === agentType);
  }

  /** Count non-terminated agents of a given type. */
  countByType(agentType: AgentType): number {
    let count = 0;
    for (const a of this.agents.values()) {
      if (a.agentType === agentType && a.status !== 'Terminated') {
        count++;
      }
    }
    return count;
  }
}
