import type { AgentType, SyscallPermission } from '@aix/shared';

// ---------------------------------------------------------------------------
// Agent status (lifecycle states)
// ---------------------------------------------------------------------------

/**
 * Lifecycle states for an agent instance.
 * Maps 1:1 to rlmx-agents AgentStatus enum.
 */
export type AgentStatus =
  | 'Pending'
  | 'Starting'
  | 'Running'
  | 'Paused'
  | 'Stopping'
  | 'Terminated'
  | 'Failed';

// ---------------------------------------------------------------------------
// Model tier
// ---------------------------------------------------------------------------

/**
 * Inference model tier selection.
 * Maps 1:1 to rlmx-agents ModelTier enum.
 */
export type ModelTier = 'Small' | 'ClaudeCode' | 'Medium' | 'Custom';

// ---------------------------------------------------------------------------
// Agent configuration
// ---------------------------------------------------------------------------

/** Configuration passed when spawning a new agent. */
export interface AgentConfig {
  /** The type of agent to spawn. */
  agentType: AgentType;
  /** Optional human-readable name. Auto-generated if omitted. */
  name?: string;
  /** Optional task description for the agent. */
  task?: string;
  /** Optional zone override. Uses preferredZone() if omitted. */
  zone?: string;
  /** Optional maximum runtime in seconds. */
  maxRuntimeSecs?: number;
}

// ---------------------------------------------------------------------------
// Agent metrics
// ---------------------------------------------------------------------------

/** Runtime metrics for an agent instance. */
export interface AgentMetrics {
  tasksCompleted: number;
  tasksFailed: number;
  uptimeSecs: number;
  memoryUsageMb: number;
  cpuUsagePct: number;
}

/** Default (zero) metrics. */
export function defaultMetrics(): AgentMetrics {
  return {
    tasksCompleted: 0,
    tasksFailed: 0,
    uptimeSecs: 0,
    memoryUsageMb: 0,
    cpuUsagePct: 0,
  };
}

// ---------------------------------------------------------------------------
// Agent info
// ---------------------------------------------------------------------------

/** Full state snapshot for an agent instance. */
export interface AgentInfo {
  id: string;
  agentType: AgentType;
  name: string;
  status: AgentStatus;
  task?: string;
  zone: string;
  spawnedAt: Date;
  parentId?: string;
  children: string[];
  modelTier: ModelTier;
  metrics: AgentMetrics;
}

// ---------------------------------------------------------------------------
// Agent type helpers (ported from Rust AgentType impl)
// ---------------------------------------------------------------------------

/** Returns the preferred model tier for a given agent type. */
export function modelTierFor(agentType: AgentType): ModelTier {
  switch (agentType) {
    case 'Coordinator':
    case 'Experimenter':
    case 'Analyst':
    case 'Trainer':
    case 'VoiceCoordinator':
    case 'MarketplaceManager':
    case 'MeshCoordinator':
    case 'FederationAgent':
      return 'Medium';
    case 'Researcher':
    case 'Reviewer':
      return 'ClaudeCode';
    default:
      return 'Small';
  }
}

/** Returns the preferred zone for a given agent type. */
export function preferredZone(agentType: AgentType): string {
  switch (agentType) {
    case 'Coordinator':
    case 'Researcher':
    case 'Reviewer':
    case 'Trainer':
    case 'Analyst':
    case 'Embedder':
    case 'VoiceCoordinator':
    case 'MeshCoordinator':
      return 'A';
    case 'Router':
    case 'Replicator':
    case 'FederationAgent':
      return 'B';
    case 'Monitor':
    case 'Validator':
      return 'C';
    default:
      return 'Multi';
  }
}

/** Returns the maximum concurrent instances for a given agent type. */
export function maxInstances(agentType: AgentType): number {
  switch (agentType) {
    case 'Coordinator':
    case 'Trainer':
      return 1;
    case 'Reviewer':
    case 'Analyst':
    case 'MarketplaceManager':
    case 'MeshCoordinator':
    case 'BillingManager':
      return 2;
    case 'Researcher':
    case 'FederationAgent':
      return 3;
    case 'VoiceCoordinator':
      return 4;
    case 'Experimenter':
      return 8;
    default:
      return 16;
  }
}

/** Returns true if the agent type can fork child processes (has ProcessFork). */
export function canFork(agentType: AgentType): boolean {
  return (
    agentType === 'Coordinator' ||
    agentType === 'Researcher' ||
    agentType === 'Experimenter' ||
    agentType === 'VoiceCoordinator' ||
    agentType === 'MeshCoordinator'
  );
}

/** Returns true if the agent type can mutate kernel state (has StateMutate). */
export function canMutateState(agentType: AgentType): boolean {
  return (
    agentType === 'Coordinator' ||
    agentType === 'Replicator' ||
    agentType === 'Experimenter' ||
    agentType === 'Trainer' ||
    agentType === 'MarketplaceManager' ||
    agentType === 'BillingManager'
  );
}
