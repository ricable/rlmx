// ---------------------------------------------------------------------------
// @aix/swarm — Zone topology
// 6 zones: A-Mobile, A-Desktop, B-Cloud, C-Edge, D-Browser, E-HomeHub
// Maps to rlmx-swarm/src/zone.rs Zone and ZoneManager.
// ---------------------------------------------------------------------------

import type { NodeId, ZoneId, ZoneConfig, NodeProfile } from './types.js';
import { ConsensusType, PlacementPolicy, newNodeId } from './types.js';
import { zoneNotFound, nodeNotFound } from './errors.js';
import type { SwarmError } from './errors.js';

/**
 * Status of a swarm node.
 */
export enum NodeStatus {
  Online = 'Online',
  Offline = 'Offline',
  Draining = 'Draining',
}

/**
 * A single node in the swarm, belonging to a zone.
 */
export interface SwarmNode {
  id: NodeId;
  zone: ZoneId;
  profile: NodeProfile;
  address: string;
  status: NodeStatus;
  lastHeartbeat: number;
}

/**
 * Create a new SwarmNode with default Online status.
 */
export function createSwarmNode(
  zone: ZoneId,
  profile: NodeProfile,
  address: string,
): SwarmNode {
  return {
    id: newNodeId(),
    zone,
    profile,
    address,
    status: NodeStatus.Online,
    lastHeartbeat: Date.now(),
  };
}

/**
 * Whether a node is considered healthy (Online or Draining).
 */
export function isNodeHealthy(node: SwarmNode): boolean {
  return node.status === NodeStatus.Online;
}

/**
 * A zone within the swarm -- a logical grouping of nodes sharing a
 * consensus type and placement policy.
 */
export interface Zone {
  id: ZoneId;
  name: string;
  consensusType: ConsensusType;
  placementPolicy: PlacementPolicy;
  nodes: Map<NodeId, SwarmNode>;
}

/**
 * Create a new empty zone.
 */
export function createZone(
  id: ZoneId,
  name: string,
  consensusType: ConsensusType,
  placementPolicy: PlacementPolicy,
): Zone {
  return { id, name, consensusType, placementPolicy, nodes: new Map() };
}

// -- Predefined 6-zone topology (CLAUDE.md) ---------------------------------

/**
 * The six canonical RLMX swarm zones per ADR-001 and CLAUDE.md.
 */
export const CANONICAL_ZONES: readonly ZoneConfig[] = [
  {
    id: 'zone-a-mobile',
    name: 'A-Mobile (Phone/Primary)',
    consensusType: ConsensusType.Raft,
    placementPolicy: PlacementPolicy.Any,
  },
  {
    id: 'zone-a-desktop',
    name: 'A-Desktop (Laptop/Secondary)',
    consensusType: ConsensusType.Raft,
    placementPolicy: PlacementPolicy.ComputeHeavy,
  },
  {
    id: 'zone-b',
    name: 'B (Cloud/Burst)',
    consensusType: ConsensusType.Pbft,
    placementPolicy: PlacementPolicy.InferenceOptimized,
  },
  {
    id: 'zone-c',
    name: 'C (Edge/Sentinel)',
    consensusType: ConsensusType.Gossip,
    placementPolicy: PlacementPolicy.EdgeRelay,
  },
  {
    id: 'zone-d',
    name: 'D (Browser)',
    consensusType: ConsensusType.Gossip,
    placementPolicy: PlacementPolicy.BrowserCompute,
  },
  {
    id: 'zone-e',
    name: 'E (HomeHub/PrivacyAnchor)',
    consensusType: ConsensusType.Raft,
    placementPolicy: PlacementPolicy.EdgeRelay,
  },
] as const;

// -- ZoneManager ------------------------------------------------------------

/**
 * Manages all zones in the swarm. Port of Rust ZoneManager.
 */
export class ZoneManager {
  readonly zones: Map<ZoneId, Zone> = new Map();

  /** Add a zone to the manager. */
  addZone(zone: Zone): void {
    this.zones.set(zone.id, zone);
  }

  /** Add a node to the specified zone. Throws SwarmError if zone not found. */
  addNode(zoneId: ZoneId, node: SwarmNode): void {
    const zone = this.zones.get(zoneId);
    if (!zone) throw zoneNotFound(zoneId);
    zone.nodes.set(node.id, node);
  }

  /** Remove a node from any zone. Throws SwarmError if not found. */
  removeNode(nodeId: NodeId): SwarmNode {
    for (const zone of this.zones.values()) {
      const node = zone.nodes.get(nodeId);
      if (node) {
        zone.nodes.delete(nodeId);
        return node;
      }
    }
    throw nodeNotFound(nodeId);
  }

  /** Get a zone by ID. */
  getZone(zoneId: ZoneId): Zone | undefined {
    return this.zones.get(zoneId);
  }

  /** Get a node by ID, searching across all zones. */
  getNode(nodeId: NodeId): SwarmNode | undefined {
    for (const zone of this.zones.values()) {
      const node = zone.nodes.get(nodeId);
      if (node) return node;
    }
    return undefined;
  }

  /** Return healthy nodes in the given zone. */
  healthyNodes(zoneId: ZoneId): SwarmNode[] {
    const zone = this.zones.get(zoneId);
    if (!zone) return [];
    return [...zone.nodes.values()].filter(isNodeHealthy);
  }

  /** Total nodes across all zones. */
  totalNodes(): number {
    let total = 0;
    for (const zone of this.zones.values()) {
      total += zone.nodes.size;
    }
    return total;
  }

  /** Find the first zone matching a placement policy. */
  findZoneForPolicy(policy: PlacementPolicy): Zone | undefined {
    for (const zone of this.zones.values()) {
      if (zone.placementPolicy === policy) return zone;
    }
    return undefined;
  }
}
