import { describe, it, expect, beforeEach } from 'vitest';
import {
  SwarmCluster,
  createSwarmConfig,
  ConsensusType,
  PlacementPolicy,
  createSwarmNode,
  type SwarmConfig,
  type ZoneConfig,
  type SwarmEvent,
} from '../src/index.js';

function testZones(): ZoneConfig[] {
  return [
    {
      id: 'z1',
      name: 'Compute',
      consensusType: ConsensusType.Pbft,
      placementPolicy: PlacementPolicy.ComputeHeavy,
    },
    {
      id: 'z2',
      name: 'Inference',
      consensusType: ConsensusType.Raft,
      placementPolicy: PlacementPolicy.InferenceOptimized,
    },
  ];
}

function testConfig(overrides?: Partial<SwarmConfig>): SwarmConfig {
  return createSwarmConfig(testZones(), {
    maxNodes: overrides?.maxNodes ?? 10,
    healthIntervalMs: 5000,
  });
}

function makeNode(zoneId: string) {
  return createSwarmNode(zoneId, {
    cpuCores: 8,
    memoryMb: 16384,
    hasGpu: false,
    architecture: 'x86_64',
  }, '127.0.0.1:9000');
}

describe('SwarmCluster', () => {
  let cluster: SwarmCluster;

  beforeEach(() => {
    cluster = new SwarmCluster(testConfig());
  });

  it('should initialize with configured zones', () => {
    expect(cluster.zones.zones.size).toBe(2);
    expect(cluster.zones.getZone('z1')).toBeDefined();
    expect(cluster.zones.getZone('z2')).toBeDefined();
  });

  it('should add a node to a zone', () => {
    const node = makeNode('z1');
    const nodeId = cluster.addNode('z1', node);
    expect(cluster.nodeCount()).toBe(1);
    expect(cluster.zones.getNode(nodeId)).toBeDefined();
  });

  it('should reject node when cluster is full', () => {
    const config = testConfig({ maxNodes: 1 });
    const c = new SwarmCluster(config);
    c.addNode('z1', makeNode('z1'));
    expect(() => c.addNode('z1', makeNode('z1'))).toThrow(/cluster full/);
  });

  it('should remove a node', () => {
    const node = makeNode('z1');
    const nodeId = cluster.addNode('z1', node);
    const removed = cluster.removeNode(nodeId);
    expect(removed.id).toBe(nodeId);
    expect(cluster.nodeCount()).toBe(0);
  });

  it('should emit NodeJoined event on add', () => {
    const events: SwarmEvent[] = [];
    cluster.eventBus.subscribe(e => events.push(e));
    cluster.addNode('z1', makeNode('z1'));
    expect(events).toHaveLength(1);
    expect(events[0].type).toBe('NodeJoined');
  });

  it('should emit NodeLeft event on remove', () => {
    const events: SwarmEvent[] = [];
    const nodeId = cluster.addNode('z1', makeNode('z1'));
    cluster.eventBus.subscribe(e => events.push(e));
    cluster.removeNode(nodeId);
    expect(events).toHaveLength(1);
    expect(events[0].type).toBe('NodeLeft');
  });

  it('should produce correct topology', () => {
    cluster.addNode('z1', makeNode('z1'));
    const topo = cluster.topology();
    expect(topo.zones).toHaveLength(2);
    expect(topo.totalNodes).toBe(1);
    // 2 zones => 1 connection (fully connected mesh)
    expect(topo.connections).toHaveLength(1);
  });

  it('should broadcast arbitrary events', () => {
    const events: SwarmEvent[] = [];
    cluster.eventBus.subscribe(e => events.push(e));
    cluster.broadcastEvent({
      type: 'ConsensusReached',
      round: 1,
      valueHash: 'abc',
    });
    expect(events).toHaveLength(1);
    if (events[0].type === 'ConsensusReached') {
      expect(events[0].round).toBe(1);
    }
  });

  it('should count healthy nodes', () => {
    const node = makeNode('z1');
    cluster.addNode('z1', node);
    // node starts Online => healthy
    expect(cluster.healthyNodeCount()).toBe(1);
  });

  it('should track nodes across multiple zones', () => {
    cluster.addNode('z1', makeNode('z1'));
    cluster.addNode('z1', makeNode('z1'));
    cluster.addNode('z2', makeNode('z2'));
    expect(cluster.nodeCount()).toBe(3);

    const topo = cluster.topology();
    const z1 = topo.zones.find(z => z.id === 'z1');
    const z2 = topo.zones.find(z => z.id === 'z2');
    expect(z1?.nodeCount).toBe(2);
    expect(z2?.nodeCount).toBe(1);
  });
});

describe('createSwarmConfig', () => {
  it('should create config with defaults', () => {
    const config = createSwarmConfig(testZones());
    expect(config.maxNodes).toBe(100);
    expect(config.healthIntervalMs).toBe(5000);
    expect(config.consensus.defaultType).toBe(ConsensusType.Raft);
    expect(config.zones).toHaveLength(2);
    expect(config.clusterId).toBeDefined();
  });

  it('should accept custom options', () => {
    const config = createSwarmConfig(testZones(), {
      maxNodes: 50,
      defaultConsensus: ConsensusType.Gossip,
    });
    expect(config.maxNodes).toBe(50);
    expect(config.consensus.defaultType).toBe(ConsensusType.Gossip);
  });
});
