import { describe, it, expect } from 'vitest';
import {
  PbftEntry,
  PbftLayer,
  RaftLayer,
  GossipLayer,
  ConsensusManager,
  ConsensusType,
  newNodeId,
} from '../src/index.js';

function makeNodes(n: number): string[] {
  return Array.from({ length: n }, () => newNodeId());
}

const enc = new TextEncoder();

describe('PbftEntry', () => {
  it('should create entry with correct digest', () => {
    const entry = new PbftEntry(1, 'StateMutate { key: v }');
    expect(entry.sequence).toBe(1);
    expect(entry.prepares.size).toBe(0);
    expect(entry.commits.size).toBe(0);
    expect(entry.verifyDigest()).toBe(true);
  });

  it('should detect tampering via digest', () => {
    const entry = new PbftEntry(1, 'StateMutate { key: v }');
    // Tamper with operation after creation
    (entry as any).operation = 'StateMutate { key: TAMPERED }';
    expect(entry.verifyDigest()).toBe(false);
  });

  it('should track prepares and commits', () => {
    const nodes = makeNodes(4);
    const entry = new PbftEntry(1, 'StateMutate');
    entry.prepares.add(nodes[0]);
    entry.prepares.add(nodes[1]);
    entry.prepares.add(nodes[2]);
    expect(entry.prepares.size).toBe(3);
    entry.commits.add(nodes[0]);
    entry.commits.add(nodes[1]);
    expect(entry.commits.size).toBe(2);
  });
});

describe('PbftLayer', () => {
  it('should require at least 4 nodes', async () => {
    const pbft = new PbftLayer(makeNodes(3));
    await expect(pbft.propose(enc.encode('value'))).rejects.toThrow(/at least 4/);
  });

  it('should succeed with 4 nodes', async () => {
    const pbft = new PbftLayer(makeNodes(4));
    expect(await pbft.propose(enc.encode('value'))).toBe(true);
  });

  it('should elect first replica as leader', async () => {
    const replicas = makeNodes(4);
    const pbft = new PbftLayer(replicas);
    expect(pbft.currentLeader()).toBe(replicas[0]);
  });

  it('should have correct defaults', () => {
    const pbft = new PbftLayer(makeNodes(5));
    expect(pbft.view).toBe(0);
    expect(pbft.checkpointInterval).toBe(100);
    expect(pbft.log).toHaveLength(0);
  });

  it('should report Pbft consensus type', () => {
    expect(new PbftLayer(makeNodes(4)).consensusType()).toBe(ConsensusType.Pbft);
  });
});

describe('RaftLayer', () => {
  it('should propose successfully with leader', async () => {
    const raft = new RaftLayer(makeNodes(3));
    expect(await raft.propose(enc.encode('val'))).toBe(true);
  });

  it('should fail without leader', async () => {
    const raft = new RaftLayer(makeNodes(3));
    raft.leader = undefined;
    await expect(raft.propose(enc.encode('val'))).rejects.toThrow(/no leader/);
  });

  it('should elect first voter as leader', () => {
    const voters = makeNodes(5);
    const raft = new RaftLayer(voters);
    expect(raft.currentLeader()).toBe(voters[0]);
  });

  it('should have correct defaults', () => {
    const raft = new RaftLayer(makeNodes(5));
    expect(raft.term).toBe(0);
    expect(raft.commitIndex).toBe(0);
    expect(raft.electionTimeoutMs).toBe(200);
    expect(raft.heartbeatIntervalMs).toBe(50);
    expect(raft.log).toHaveLength(0);
  });

  it('should report Raft consensus type', () => {
    expect(new RaftLayer(makeNodes(3)).consensusType()).toBe(ConsensusType.Raft);
  });
});

describe('GossipLayer', () => {
  it('should propagate state updates', () => {
    const nodes = makeNodes(3);
    const gossip = new GossipLayer(nodes, 2);
    gossip.updateState(nodes[0], 'cpu', 0.5);
    const state = gossip.getState(nodes[0]);
    expect(state).toBeDefined();
    expect(state!.generation).toBe(1);
    expect(state!.data['cpu']).toBe(0.5);
  });

  it('should propose successfully with members', async () => {
    const gossip = new GossipLayer(makeNodes(3), 2);
    expect(await gossip.propose(enc.encode('data'))).toBe(true);
  });

  it('should have no leader', () => {
    const gossip = new GossipLayer(makeNodes(3), 2);
    expect(gossip.currentLeader()).toBeUndefined();
  });

  it('should have correct defaults', () => {
    const gossip = new GossipLayer(makeNodes(3), 3);
    expect(gossip.intervalMs).toBe(500);
    expect(gossip.suspicionTimeoutMs).toBe(3000);
    expect(gossip.deadTimeoutMs).toBe(10000);
    expect(gossip.metricsBuffer).toHaveLength(0);
  });

  it('should buffer metrics with cap at 1000', () => {
    const nodes = makeNodes(3);
    const gossip = new GossipLayer(nodes, 2);
    gossip.pushMetrics({
      node: nodes[0],
      timestamp: new Date().toISOString(),
      load: 0.8,
      memoryPressure: 0.4,
      inferenceLatencyP99Ms: 55,
      activeProcesses: 3,
      vectorSegments: 7,
    });
    expect(gossip.metricsBuffer).toHaveLength(1);
    expect(gossip.metricsBuffer[0].load).toBeCloseTo(0.8);
    expect(gossip.metricsBuffer[0].vectorSegments).toBe(7);
  });

  it('should report Gossip consensus type', () => {
    expect(new GossipLayer(makeNodes(3), 2).consensusType()).toBe(ConsensusType.Gossip);
  });
});

describe('ConsensusManager', () => {
  it('should route proposals to registered layers', async () => {
    const mgr = new ConsensusManager();
    mgr.register(new RaftLayer(makeNodes(3)));
    mgr.register(new GossipLayer(makeNodes(3), 2));

    expect(await mgr.propose(ConsensusType.Raft, enc.encode('v'))).toBe(true);
    expect(await mgr.propose(ConsensusType.Gossip, enc.encode('v'))).toBe(true);
    await expect(mgr.propose(ConsensusType.Pbft, enc.encode('v'))).rejects.toThrow(/no layer/);
  });

  it('should route syscalls to correct consensus type', () => {
    const mgr = new ConsensusManager();
    expect(mgr.submitForSyscall('StateMutate')).toBe(ConsensusType.Pbft);
    expect(mgr.submitForSyscall('ClusterMembership')).toBe(ConsensusType.Raft);
    expect(mgr.submitForSyscall('ConfigChange')).toBe(ConsensusType.Raft);
    expect(mgr.submitForSyscall('TokenRevocation')).toBe(ConsensusType.Raft);
    expect(mgr.submitForSyscall('VecSearch')).toBe(ConsensusType.Gossip);
    expect(mgr.submitForSyscall('GraphQuery')).toBe(ConsensusType.Gossip);
    expect(mgr.submitForSyscall('ProcessFork')).toBe(ConsensusType.Gossip);
  });

  it('should return leader for registered layer', () => {
    const mgr = new ConsensusManager();
    const voters = makeNodes(3);
    mgr.register(new RaftLayer(voters));
    expect(mgr.leader(ConsensusType.Raft)).toBe(voters[0]);
    expect(mgr.leader(ConsensusType.Pbft)).toBeUndefined();
  });
});
