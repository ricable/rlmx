// ---------------------------------------------------------------------------
// @aix/swarm — SwarmCluster aggregate root
// Port of rlmx-swarm/src/cluster.rs.
// ---------------------------------------------------------------------------

import type { ClusterId, NodeId, ZoneId, SwarmConfig, ZoneConfig } from './types.js';
import { ConsensusType, newClusterId } from './types.js';
import { SwarmEventBus } from './events.js';
import type { SwarmEvent } from './events.js';
import {
  ZoneManager,
  createZone,
  type SwarmNode,
} from './zones.js';
import { clusterFull } from './errors.js';

// -- Topology snapshot types ------------------------------------------------

/** Info about a single zone. */
export interface ZoneInfo {
  id: ZoneId;
  name: string;
  nodeCount: number;
  consensusType: ConsensusType;
}

/** A connection between two zones. */
export interface Connection {
  from: ZoneId;
  to: ZoneId;
  latencyMs: number;
}

/** Serializable snapshot of the cluster topology. */
export interface SwarmTopology {
  zones: ZoneInfo[];
  totalNodes: number;
  healthyNodes: number;
  connections: Connection[];
}

// -- SwarmCluster -----------------------------------------------------------

/**
 * The top-level swarm cluster, managing zones and broadcasting events.
 * Aggregate root for the Swarm Coordination bounded context.
 */
export class SwarmCluster {
  readonly id: ClusterId;
  readonly config: SwarmConfig;
  readonly zones: ZoneManager;
  readonly eventBus: SwarmEventBus;

  constructor(config: SwarmConfig) {
    this.id = config.clusterId;
    this.config = config;
    this.zones = new ZoneManager();
    this.eventBus = new SwarmEventBus();

    // Initialize zones from config
    for (const zc of config.zones) {
      this.zones.addZone(
        createZone(zc.id, zc.name, zc.consensusType, zc.placementPolicy),
      );
    }
  }

  /** Add a node to the specified zone. */
  addNode(zoneId: ZoneId, node: SwarmNode): NodeId {
    if (this.zones.totalNodes() >= this.config.maxNodes) {
      throw clusterFull(this.config.maxNodes);
    }
    const nodeId = node.id;
    this.zones.addNode(zoneId, node);
    this.eventBus.emit({ type: 'NodeJoined', nodeId, zoneId });
    return nodeId;
  }

  /** Remove a node from the cluster. */
  removeNode(nodeId: NodeId): SwarmNode {
    const node = this.zones.removeNode(nodeId);
    this.eventBus.emit({ type: 'NodeLeft', nodeId, zoneId: node.zone });
    return node;
  }

  /** Get a snapshot of the cluster topology. */
  topology(): SwarmTopology {
    const zoneInfos: ZoneInfo[] = [];
    const zoneIds: ZoneId[] = [];

    for (const [id, zone] of this.zones.zones) {
      zoneInfos.push({
        id,
        name: zone.name,
        nodeCount: zone.nodes.size,
        consensusType: zone.consensusType,
      });
      zoneIds.push(id);
    }

    // Fully connected mesh between zones
    const connections: Connection[] = [];
    for (let i = 0; i < zoneIds.length; i++) {
      for (let j = i + 1; j < zoneIds.length; j++) {
        connections.push({
          from: zoneIds[i],
          to: zoneIds[j],
          latencyMs: 1, // simulated
        });
      }
    }

    let healthyNodes = 0;
    for (const id of zoneIds) {
      healthyNodes += this.zones.healthyNodes(id).length;
    }

    return {
      zones: zoneInfos,
      totalNodes: this.zones.totalNodes(),
      healthyNodes,
      connections,
    };
  }

  /** Broadcast an event to all subscribers. */
  broadcastEvent(event: SwarmEvent): void {
    this.eventBus.emit(event);
  }

  /** Total number of nodes in the cluster. */
  nodeCount(): number {
    return this.zones.totalNodes();
  }

  /** Number of healthy nodes in the cluster. */
  healthyNodeCount(): number {
    let count = 0;
    for (const id of this.zones.zones.keys()) {
      count += this.zones.healthyNodes(id).length;
    }
    return count;
  }
}

// -- Factory ----------------------------------------------------------------

/**
 * Create a SwarmConfig with sensible defaults.
 */
export function createSwarmConfig(
  zones: ZoneConfig[],
  options?: {
    clusterId?: ClusterId;
    maxNodes?: number;
    healthIntervalMs?: number;
    defaultConsensus?: ConsensusType;
  },
): SwarmConfig {
  return {
    clusterId: options?.clusterId ?? newClusterId(),
    zones,
    maxNodes: options?.maxNodes ?? 100,
    consensus: {
      zoneConfigs: {},
      defaultType: options?.defaultConsensus ?? ConsensusType.Raft,
    },
    healthIntervalMs: options?.healthIntervalMs ?? 5000,
  };
}
