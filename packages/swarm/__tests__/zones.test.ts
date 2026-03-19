import { describe, it, expect, beforeEach } from 'vitest';
import {
  ZoneManager,
  createZone,
  createSwarmNode,
  isNodeHealthy,
  NodeStatus,
  ConsensusType,
  PlacementPolicy,
  CANONICAL_ZONES,
  type SwarmNode,
} from '../src/index.js';

function makeZone(id: string, name: string, policy: PlacementPolicy) {
  return createZone(id, name, ConsensusType.Raft, policy);
}

function makeNode(zoneId: string): SwarmNode {
  return createSwarmNode(zoneId, {
    cpuCores: 4,
    memoryMb: 8192,
    hasGpu: false,
    architecture: 'x86_64',
  }, '127.0.0.1:9000');
}

describe('CANONICAL_ZONES', () => {
  it('should define exactly 6 zones', () => {
    expect(CANONICAL_ZONES).toHaveLength(6);
  });

  it('should include all required zone types', () => {
    const ids = CANONICAL_ZONES.map(z => z.id);
    expect(ids).toContain('zone-a-mobile');
    expect(ids).toContain('zone-a-desktop');
    expect(ids).toContain('zone-b');
    expect(ids).toContain('zone-c');
    expect(ids).toContain('zone-d');
    expect(ids).toContain('zone-e');
  });

  it('should assign correct consensus types', () => {
    const byId = new Map(CANONICAL_ZONES.map(z => [z.id, z]));
    expect(byId.get('zone-a-mobile')!.consensusType).toBe(ConsensusType.Raft);
    expect(byId.get('zone-b')!.consensusType).toBe(ConsensusType.Pbft);
    expect(byId.get('zone-c')!.consensusType).toBe(ConsensusType.Gossip);
    expect(byId.get('zone-d')!.consensusType).toBe(ConsensusType.Gossip);
  });

  it('should assign correct placement policies', () => {
    const byId = new Map(CANONICAL_ZONES.map(z => [z.id, z]));
    expect(byId.get('zone-a-desktop')!.placementPolicy).toBe(PlacementPolicy.ComputeHeavy);
    expect(byId.get('zone-b')!.placementPolicy).toBe(PlacementPolicy.InferenceOptimized);
    expect(byId.get('zone-d')!.placementPolicy).toBe(PlacementPolicy.BrowserCompute);
  });
});

describe('isNodeHealthy', () => {
  it('should return true for Online nodes', () => {
    const node = makeNode('z1');
    expect(isNodeHealthy(node)).toBe(true);
  });

  it('should return false for Offline nodes', () => {
    const node = makeNode('z1');
    node.status = NodeStatus.Offline;
    expect(isNodeHealthy(node)).toBe(false);
  });

  it('should return false for Draining nodes', () => {
    const node = makeNode('z1');
    node.status = NodeStatus.Draining;
    expect(isNodeHealthy(node)).toBe(false);
  });
});

describe('ZoneManager', () => {
  let mgr: ZoneManager;

  beforeEach(() => {
    mgr = new ZoneManager();
  });

  it('should add and get a zone', () => {
    mgr.addZone(makeZone('z1', 'Zone A', PlacementPolicy.Any));
    expect(mgr.getZone('z1')).toBeDefined();
    expect(mgr.getZone('z2')).toBeUndefined();
  });

  it('should add a node to a zone', () => {
    mgr.addZone(makeZone('z1', 'Zone A', PlacementPolicy.Any));
    const node = makeNode('z1');
    mgr.addNode('z1', node);
    expect(mgr.getNode(node.id)).toBeDefined();
  });

  it('should throw when adding to missing zone', () => {
    const node = makeNode('z1');
    expect(() => mgr.addNode('z999', node)).toThrow(/zone not found/i);
  });

  it('should remove a node', () => {
    mgr.addZone(makeZone('z1', 'Zone A', PlacementPolicy.Any));
    const node = makeNode('z1');
    mgr.addNode('z1', node);
    const removed = mgr.removeNode(node.id);
    expect(removed.id).toBe(node.id);
    expect(mgr.getNode(node.id)).toBeUndefined();
  });

  it('should throw when removing nonexistent node', () => {
    mgr.addZone(makeZone('z1', 'Zone A', PlacementPolicy.Any));
    expect(() => mgr.removeNode('nonexistent')).toThrow(/node not found/i);
  });

  it('should filter healthy nodes', () => {
    mgr.addZone(makeZone('z1', 'Zone A', PlacementPolicy.Any));
    const n1 = makeNode('z1'); // Online
    const n2 = makeNode('z1');
    n2.status = NodeStatus.Offline;
    mgr.addNode('z1', n1);
    mgr.addNode('z1', n2);
    expect(mgr.healthyNodes('z1')).toHaveLength(1);
  });

  it('should count total nodes across zones', () => {
    mgr.addZone(makeZone('z1', 'Zone A', PlacementPolicy.Any));
    mgr.addZone(makeZone('z2', 'Zone B', PlacementPolicy.ComputeHeavy));
    mgr.addNode('z1', makeNode('z1'));
    mgr.addNode('z1', makeNode('z1'));
    mgr.addNode('z2', makeNode('z2'));
    expect(mgr.totalNodes()).toBe(3);
  });

  it('should find zone for placement policy', () => {
    mgr.addZone(makeZone('z1', 'Zone A', PlacementPolicy.ComputeHeavy));
    mgr.addZone(makeZone('z2', 'Zone B', PlacementPolicy.EdgeRelay));

    const found = mgr.findZoneForPolicy(PlacementPolicy.EdgeRelay);
    expect(found).toBeDefined();
    expect(found!.id).toBe('z2');

    expect(mgr.findZoneForPolicy(PlacementPolicy.InferenceOptimized)).toBeUndefined();
  });

  it('should initialize with canonical zones', () => {
    for (const zc of CANONICAL_ZONES) {
      mgr.addZone(createZone(zc.id, zc.name, zc.consensusType, zc.placementPolicy));
    }
    expect(mgr.zones.size).toBe(6);
    expect(mgr.totalNodes()).toBe(0);
  });
});
