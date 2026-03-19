import { AgentType, SyscallPermission, AGENT_TYPES, SYSCALL_PERMISSIONS } from '@aix/shared';

// ---------------------------------------------------------------------------
// AgentPermissions — a mutable permission set
// ---------------------------------------------------------------------------

/**
 * A set of syscall permissions granted to an agent.
 * Wraps a Set<SyscallPermission> with convenience methods.
 */
export class AgentPermissions {
  private readonly permissions: Set<SyscallPermission>;

  constructor(perms: Iterable<SyscallPermission> = []) {
    this.permissions = new Set(perms);
  }

  /** Check whether a permission is granted. */
  has(perm: SyscallPermission): boolean {
    return this.permissions.has(perm);
  }

  /** Grant a permission. */
  grant(perm: SyscallPermission): void {
    this.permissions.add(perm);
  }

  /** Revoke a permission. */
  revoke(perm: SyscallPermission): void {
    this.permissions.delete(perm);
  }

  /** Number of granted permissions. */
  get count(): number {
    return this.permissions.size;
  }

  /** Iterate over granted permissions. */
  [Symbol.iterator](): IterableIterator<SyscallPermission> {
    return this.permissions[Symbol.iterator]();
  }

  /** Return a frozen array copy of the permission set. */
  toArray(): readonly SyscallPermission[] {
    return Object.freeze([...this.permissions]);
  }
}

// ---------------------------------------------------------------------------
// Permission matrix — static data (Map<AgentType, SyscallPermission[]>)
// ---------------------------------------------------------------------------

const P = SyscallPermission;

/**
 * The 17x17 permission matrix.
 * Each entry maps an AgentType to its allowed SyscallPermissions.
 * This is pure DATA, not computed at runtime.
 */
const PERMISSION_MATRIX: ReadonlyMap<AgentType, readonly SyscallPermission[]> = new Map<
  AgentType,
  readonly SyscallPermission[]
>([
  // Coordinator: full access (PID 0)
  [
    AgentType.Coordinator,
    SYSCALL_PERMISSIONS,
  ],

  // Researcher: read vectors, graph queries, fork, messaging, attention, voice synth, intent, artifact write
  [
    AgentType.Researcher,
    [
      P.VecInsert, P.VecSearch,
      P.GraphQuery, P.GraphDiffuse,
      P.ProcessFork, P.ProcessSend, P.ProcessRecv,
      P.AttentionSelect,
      P.VoiceSynthesize, P.IntentRoute,
      P.ArtifactWrite,
    ],
  ],

  // Router: read-only routing, messaging, attention, halt, voice transcribe/synth, intent
  [
    AgentType.Router,
    [
      P.VecSearch,
      P.ProcessSend, P.ProcessRecv,
      P.AttentionSelect, P.HaltCheck,
      P.VoiceTranscribe, P.VoiceSynthesize, P.IntentRoute,
    ],
  ],

  // Experimenter: broad access for experiments, can fork workers, artifact write
  [
    AgentType.Experimenter,
    [
      P.VecInsert, P.VecSearch, P.VecDelete,
      P.GraphQuery, P.GraphDiffuse,
      P.ProcessFork, P.ProcessSend, P.ProcessRecv,
      P.StateMutate, P.AttentionSelect,
      P.VoiceSynthesize, P.IntentRoute,
      P.ArtifactWrite,
    ],
  ],

  // Worker: execute tasks, messaging, state mutation, voice synth, intent
  [
    AgentType.Worker,
    [
      P.VecInsert, P.VecSearch, P.VecDelete,
      P.ProcessSend, P.ProcessRecv,
      P.StateMutate,
      P.VoiceSynthesize, P.IntentRoute,
    ],
  ],

  // Monitor: read-only observation, messaging, halt check, voice synth, intent
  [
    AgentType.Monitor,
    [
      P.VecSearch, P.GraphQuery,
      P.ProcessSend, P.ProcessRecv,
      P.AttentionSelect, P.HaltCheck,
      P.VoiceSynthesize, P.IntentRoute,
    ],
  ],

  // Reviewer: read access, messaging, attention, state mutate, voice synth, intent
  [
    AgentType.Reviewer,
    [
      P.VecSearch,
      P.GraphQuery, P.GraphDiffuse,
      P.ProcessSend, P.ProcessRecv,
      P.AttentionSelect, P.StateMutate,
      P.VoiceSynthesize, P.IntentRoute,
    ],
  ],

  // Trainer: vector ops, state mutation, messaging, voice synth, intent
  [
    AgentType.Trainer,
    [
      P.VecInsert, P.VecSearch, P.VecDelete,
      P.ProcessSend, P.ProcessRecv,
      P.StateMutate, P.AttentionSelect,
      P.VoiceSynthesize, P.IntentRoute,
    ],
  ],

  // Validator: read-only verification, no state mutation, voice synth, intent
  [
    AgentType.Validator,
    [
      P.VecSearch, P.GraphQuery,
      P.ProcessSend, P.ProcessRecv,
      P.HaltCheck,
      P.VoiceSynthesize, P.IntentRoute,
    ],
  ],

  // Replicator: vector ops for sync, messaging, voice synth, intent
  [
    AgentType.Replicator,
    [
      P.VecInsert, P.VecSearch, P.VecDelete,
      P.ProcessSend, P.ProcessRecv,
      P.StateMutate,
      P.VoiceSynthesize, P.IntentRoute,
    ],
  ],

  // Embedder: vector insert/search only, no state mutation, voice synth, intent
  [
    AgentType.Embedder,
    [
      P.VecInsert, P.VecSearch,
      P.ProcessSend, P.ProcessRecv,
      P.VoiceSynthesize, P.IntentRoute,
    ],
  ],

  // Analyst: graph operations, vector search, attention, state mutate, voice synth, intent
  [
    AgentType.Analyst,
    [
      P.VecSearch,
      P.GraphQuery, P.GraphCut, P.GraphDiffuse,
      P.ProcessSend, P.ProcessRecv,
      P.AttentionSelect, P.StateMutate,
      P.VoiceSynthesize, P.IntentRoute,
    ],
  ],

  // VoiceCoordinator: full voice access, fork, messaging, attention, intent
  [
    AgentType.VoiceCoordinator,
    [
      P.VecInsert, P.VecSearch,
      P.ProcessFork, P.ProcessSend, P.ProcessRecv,
      P.AttentionSelect,
      P.VoiceTranscribe, P.VoiceSynthesize, P.IntentRoute,
    ],
  ],

  // MarketplaceManager: state mutation, vector ops, messaging, voice synth, intent
  [
    AgentType.MarketplaceManager,
    [
      P.VecInsert, P.VecSearch, P.VecDelete,
      P.ProcessSend, P.ProcessRecv,
      P.StateMutate,
      P.VoiceSynthesize, P.IntentRoute,
    ],
  ],

  // MeshCoordinator: graph, vectors, fork, messaging, attention, mesh sync, voice synth, intent
  [
    AgentType.MeshCoordinator,
    [
      P.VecInsert, P.VecSearch,
      P.GraphQuery, P.GraphDiffuse,
      P.ProcessFork, P.ProcessSend, P.ProcessRecv,
      P.AttentionSelect, P.MeshSync,
      P.VoiceSynthesize, P.IntentRoute,
    ],
  ],

  // FederationAgent: search, messaging, attention, federation, voice synth, intent
  [
    AgentType.FederationAgent,
    [
      P.VecSearch,
      P.ProcessSend, P.ProcessRecv,
      P.AttentionSelect, P.FederationContribute,
      P.VoiceSynthesize, P.IntentRoute,
    ],
  ],

  // BillingManager: vector ops, messaging, state mutation, voice synth, intent
  [
    AgentType.BillingManager,
    [
      P.VecInsert, P.VecSearch, P.VecDelete,
      P.ProcessSend, P.ProcessRecv,
      P.StateMutate,
      P.VoiceSynthesize, P.IntentRoute,
    ],
  ],
]);

// ---------------------------------------------------------------------------
// Spawn hierarchy — static data
// ---------------------------------------------------------------------------

/**
 * Valid parent -> children spawn relationships.
 * If an agent type is not in this map, it cannot spawn any children.
 */
const SPAWN_HIERARCHY: ReadonlyMap<AgentType, readonly AgentType[]> = new Map([
  // Coordinator can spawn anything except another Coordinator
  [
    AgentType.Coordinator,
    AGENT_TYPES.filter((t) => t !== AgentType.Coordinator),
  ],
  // Researcher can spawn Workers, Experimenters, and Embedders
  [
    AgentType.Researcher,
    [AgentType.Worker, AgentType.Experimenter, AgentType.Embedder],
  ],
  // Experimenter can spawn Workers
  [AgentType.Experimenter, [AgentType.Worker]],
  // VoiceCoordinator can spawn Workers and Embedders
  [AgentType.VoiceCoordinator, [AgentType.Worker, AgentType.Embedder]],
  // MeshCoordinator can spawn Workers
  [AgentType.MeshCoordinator, [AgentType.Worker]],
]);

// ---------------------------------------------------------------------------
// PermissionRegistry — static facade
// ---------------------------------------------------------------------------

/**
 * Static permission registry mapping each AgentType to allowed syscalls.
 * Port of rlmx-agents PermissionRegistry.
 */
export class PermissionRegistry {
  private constructor() {}

  /**
   * Returns the permissions for a given agent type.
   * This is the 17x17 matrix lookup.
   */
  static permissionsFor(agentType: AgentType): AgentPermissions {
    const perms = PERMISSION_MATRIX.get(agentType);
    if (!perms) {
      return new AgentPermissions();
    }
    return new AgentPermissions(perms);
  }

  /**
   * Validates that an agent of the given type has a specific permission.
   * Returns true if the permission is in the agent's set.
   */
  static validate(agentType: AgentType, permission: SyscallPermission): boolean {
    // All wildcard grants everything
    if (permission === SyscallPermission.All) {
      return true;
    }
    return PermissionRegistry.permissionsFor(agentType).has(permission);
  }

  /**
   * Whether `parent` can spawn `child` based on the hierarchy.
   */
  static canSpawn(parent: AgentType, child: AgentType): boolean {
    const children = SPAWN_HIERARCHY.get(parent);
    return children !== undefined && children.includes(child);
  }

  /**
   * Returns the valid children that `parent` can spawn.
   */
  static spawnableChildren(parent: AgentType): readonly AgentType[] {
    return SPAWN_HIERARCHY.get(parent) ?? [];
  }
}
