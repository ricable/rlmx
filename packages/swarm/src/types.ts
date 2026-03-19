// ---------------------------------------------------------------------------
// @aix/swarm — Core types
// Maps to rlmx-swarm/src/types.rs: NodeId, ZoneId, ClusterId, SwarmConfig,
// ConsensusType, PlacementPolicy, MetricsUpdate, NodeProfile, etc.
// ---------------------------------------------------------------------------

import { randomUUID } from 'node:crypto';

/** Unique identifier for a swarm node. */
export type NodeId = string;

/** Generate a new unique NodeId. */
export function newNodeId(): NodeId {
  return randomUUID();
}

/** Identifier for a zone within the swarm. */
export type ZoneId = string;

/** Unique identifier for a cluster. */
export type ClusterId = string;

/** Generate a new unique ClusterId. */
export function newClusterId(): ClusterId {
  return randomUUID();
}

/**
 * Type of consensus protocol used within a zone.
 * Maps 1:1 to Rust ConsensusType enum.
 */
export enum ConsensusType {
  Pbft = 'Pbft',
  Raft = 'Raft',
  Gossip = 'Gossip',
}

/**
 * Node placement policy for a zone.
 * Maps 1:1 to Rust PlacementPolicy enum.
 */
export enum PlacementPolicy {
  ComputeHeavy = 'ComputeHeavy',
  InferenceOptimized = 'InferenceOptimized',
  EdgeRelay = 'EdgeRelay',
  /** Browser WASM compute workers (ADR-009). Stateless, no consensus participation. */
  BrowserCompute = 'BrowserCompute',
  Any = 'Any',
}

/** Hardware profile of a swarm node. */
export interface NodeProfile {
  cpuCores: number;
  memoryMb: number;
  hasGpu: boolean;
  gpuType?: string;
  architecture: string;
}

/** Configuration for a single zone. */
export interface ZoneConfig {
  id: ZoneId;
  name: string;
  consensusType: ConsensusType;
  placementPolicy: PlacementPolicy;
}

/** Consensus configuration mapping zones to consensus types. */
export interface ConsensusConfig {
  zoneConfigs: Record<ZoneId, ConsensusType>;
  defaultType: ConsensusType;
}

/** Configuration for the entire swarm. */
export interface SwarmConfig {
  clusterId: ClusterId;
  zones: ZoneConfig[];
  maxNodes: number;
  consensus: ConsensusConfig;
  healthIntervalMs: number;
}

/** Health and performance metrics update disseminated via gossip. */
export interface MetricsUpdate {
  node: NodeId;
  timestamp: string;
  load: number;
  memoryPressure: number;
  inferenceLatencyP99Ms: number;
  activeProcesses: number;
  vectorSegments: number;
}

/**
 * Maps kernel Strategy names to their preferred zone placement order.
 * Each strategy maps to a list of zone IDs where the first is the primary
 * zone and subsequent entries are fallback zones, matching ADR-001.
 */
export class StrategyZoneMapping {
  private mappings: Map<string, string[]>;

  constructor(mappings?: Map<string, string[]>) {
    this.mappings = mappings ?? new Map();
  }

  /** Create the default mapping per ADR-001 placement policy table. */
  static defaultMapping(): StrategyZoneMapping {
    const m = new Map<string, string[]>();
    m.set('Rlm', ['zone-a', 'zone-d']);
    m.set('Trm', ['zone-a', 'zone-b']);
    m.set('Edge', ['zone-b', 'zone-c']);
    m.set('Hybrid', ['zone-a']);
    m.set('Swarm', ['zone-a', 'zone-b', 'zone-c']);
    return new StrategyZoneMapping(m);
  }

  /** Return the ordered list of zone IDs for the given strategy. */
  zonesForStrategy(strategy: string): string[] {
    return this.mappings.get(strategy) ?? [];
  }
}
