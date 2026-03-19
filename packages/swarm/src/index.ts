// ---------------------------------------------------------------------------
// @aix/swarm — Swarm orchestration for the @aix ecosystem
// ---------------------------------------------------------------------------

// Types
export type {
  NodeId,
  ZoneId,
  ClusterId,
  SwarmConfig,
  ZoneConfig,
  ConsensusConfig,
  NodeProfile,
  MetricsUpdate,
} from './types.js';

export {
  ConsensusType,
  PlacementPolicy,
  StrategyZoneMapping,
  newNodeId,
  newClusterId,
} from './types.js';

// Errors
export {
  SwarmErrorCode,
  SwarmError,
  nodeNotFound,
  zoneNotFound,
  clusterFull,
  consensusFailed,
  sandboxNotFound,
  profileNotFound,
  invalidTransition,
  duplicateProfile,
  fleetValidation,
} from './errors.js';

// Events
export type {
  CardData,
  NodeJoinedEvent,
  NodeLeftEvent,
  HealthUpdateEvent,
  AgentSpawnedEvent,
  AgentTerminatedEvent,
  ConsensusReachedEvent,
  ExperimentUpdateEvent,
  MutationFoundEvent,
  SandboxSpawnedEvent,
  SandboxTerminatedEvent,
  VoiceChunkEvent,
  AgentProgressEvent,
  MultimodalResponseEvent,
  SwarmEvent,
  SwarmEventType,
  SwarmEventHandler,
} from './events.js';

export { HapticPattern, SWARM_EVENT_TYPES, SwarmEventBus } from './events.js';

// Zones
export type { SwarmNode, Zone } from './zones.js';

export {
  NodeStatus,
  createSwarmNode,
  isNodeHealthy,
  createZone,
  CANONICAL_ZONES,
  ZoneManager,
} from './zones.js';

// Consensus
export type { ConsensusLayer, RaftEntry, GossipState } from './consensus.js';

export {
  PbftEntry,
  PbftLayer,
  RaftLayer,
  GossipLayer,
  ConsensusManager,
} from './consensus.js';

// Sandbox
export type {
  ResourceEnvelope,
  SandboxProfile,
  SandboxId,
  SandboxMetrics,
  SandboxInstance,
  SandboxSpec,
  FleetManifest,
} from './sandbox.js';

export {
  GpuRequirement,
  NetworkPolicy,
  SandboxState,
  defaultMetrics,
  validTransition,
  SandboxManager,
} from './sandbox.js';

// Fleet
export type { FleetStatus } from './fleet.js';
export { FleetOrchestrator } from './fleet.js';

// Browser Pool
export type {
  BrowserCapabilities,
  BrowserWorker,
  ComputeTaskType,
  ComputeTask,
  InFlightEntry,
} from './browser-pool.js';

export {
  WorkerStatus,
  defaultCapabilities,
  BrowserComputePool,
  createComputeTask,
} from './browser-pool.js';

// Cluster
export type { ZoneInfo, Connection, SwarmTopology } from './cluster.js';
export { SwarmCluster, createSwarmConfig } from './cluster.js';
