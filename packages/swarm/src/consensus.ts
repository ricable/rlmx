// ---------------------------------------------------------------------------
// @aix/swarm — ConsensusLayer (PBFT, Raft, Gossip strategies)
// Port of rlmx-swarm/src/consensus.rs.
//
// Consensus verification uses SHA-256 via Node.js crypto (or @aix/core
// napiDispatch when available).
// ---------------------------------------------------------------------------

import { createHash, randomUUID } from 'node:crypto';

import type { NodeId, MetricsUpdate } from './types.js';
import { ConsensusType, newNodeId } from './types.js';
import { consensusFailed } from './errors.js';

// -- ConsensusLayer interface ------------------------------------------------

/**
 * Trait-equivalent for consensus protocol implementations.
 */
export interface ConsensusLayer {
  /** Propose a value for consensus. Returns true if accepted. */
  propose(value: Uint8Array): Promise<boolean>;
  /** Get the current leader, if any. */
  currentLeader(): NodeId | undefined;
  /** The consensus type this layer implements. */
  consensusType(): ConsensusType;
}

// -- PBFT -------------------------------------------------------------------

/** A single PBFT log entry with digest verification. */
export class PbftEntry {
  readonly sequence: number;
  readonly operation: string;
  readonly witnessId: string;
  readonly digest: Uint8Array;
  readonly prepares: Set<NodeId> = new Set();
  readonly commits: Set<NodeId> = new Set();

  constructor(sequence: number, operation: string, witnessId?: string) {
    this.sequence = sequence;
    this.operation = operation;
    this.witnessId = witnessId ?? randomUUID();
    this.digest = sha256(operation);
  }

  /** Verify the digest matches the operation. */
  verifyDigest(): boolean {
    const computed = sha256(this.operation);
    return buffersEqual(this.digest, computed);
  }
}

/**
 * PBFT consensus layer. Requires at least 4 replicas (3f+1).
 */
export class PbftLayer implements ConsensusLayer {
  readonly replicas: NodeId[];
  leader: NodeId | undefined;
  view = 0;
  readonly log: PbftEntry[] = [];
  checkpointInterval = 100;

  constructor(replicas: NodeId[]) {
    this.replicas = replicas;
    this.leader = replicas[0];
  }

  private maxFaults(): number {
    if (this.replicas.length < 4) return 0;
    return Math.floor((this.replicas.length - 1) / 3);
  }

  private quorumSize(): number {
    return 2 * this.maxFaults() + 1;
  }

  async propose(_value: Uint8Array): Promise<boolean> {
    const n = this.replicas.length;
    if (n < 4) {
      throw consensusFailed(`PBFT requires at least 4 nodes, have ${n}`);
    }
    return n >= this.quorumSize();
  }

  currentLeader(): NodeId | undefined {
    return this.leader;
  }

  consensusType(): ConsensusType {
    return ConsensusType.Pbft;
  }
}

// -- Raft -------------------------------------------------------------------

/** Raft log entry variants. */
export type RaftEntry =
  | { type: 'ClusterMembership'; value: string }
  | { type: 'PlacementPolicyUpdate'; value: string }
  | { type: 'TokenRevocation'; tokenId: string }
  | { type: 'ConfigChange'; value: string }
  | { type: 'ZoneReassignment'; node: NodeId; fromZone: string; toZone: string };

/**
 * Raft consensus layer with leader election semantics.
 */
export class RaftLayer implements ConsensusLayer {
  readonly voters: NodeId[];
  leader: NodeId | undefined;
  term = 0;
  votedFor: NodeId | undefined;
  readonly log: RaftEntry[] = [];
  commitIndex = 0;
  electionTimeoutMs = 200;
  heartbeatIntervalMs = 50;

  constructor(voters: NodeId[]) {
    this.voters = voters;
    this.leader = voters[0];
    this.votedFor = this.leader;
  }

  private majority(): number {
    return Math.floor(this.voters.length / 2) + 1;
  }

  async propose(_value: Uint8Array): Promise<boolean> {
    if (this.leader === undefined) {
      throw consensusFailed('no leader elected');
    }
    return this.voters.length >= this.majority();
  }

  currentLeader(): NodeId | undefined {
    return this.leader;
  }

  consensusType(): ConsensusType {
    return ConsensusType.Raft;
  }
}

// -- Gossip -----------------------------------------------------------------

/** State tracked per gossip member. */
export interface GossipState {
  generation: number;
  data: Record<string, unknown>;
  lastUpdate: string;
}

/**
 * Gossip-based eventual consistency layer.
 */
export class GossipLayer implements ConsensusLayer {
  readonly members: Map<NodeId, GossipState>;
  readonly fanout: number;
  intervalMs = 500;
  suspicionTimeoutMs = 3000;
  deadTimeoutMs = 10000;
  readonly metricsBuffer: MetricsUpdate[] = [];

  constructor(nodeIds: NodeId[], fanout: number) {
    this.fanout = fanout;
    this.members = new Map();
    for (const id of nodeIds) {
      this.members.set(id, {
        generation: 0,
        data: {},
        lastUpdate: new Date().toISOString(),
      });
    }
  }

  /** Update state for a specific node. */
  updateState(nodeId: NodeId, key: string, value: unknown): void {
    const state = this.members.get(nodeId);
    if (state) {
      state.generation += 1;
      state.data[key] = value;
      state.lastUpdate = new Date().toISOString();
    }
  }

  /** Get state for a specific node. */
  getState(nodeId: NodeId): GossipState | undefined {
    return this.members.get(nodeId);
  }

  /** Push a metrics update to the buffer. Caps at 1000, batch-evicts to avoid O(n) shift. */
  pushMetrics(update: MetricsUpdate): void {
    if (this.metricsBuffer.length >= 1000) {
      // Drop oldest 100 in one splice instead of shift() per item
      this.metricsBuffer.splice(0, 100);
    }
    this.metricsBuffer.push(update);
  }

  async propose(_value: Uint8Array): Promise<boolean> {
    return this.members.size > 0;
  }

  currentLeader(): NodeId | undefined {
    return undefined; // gossip is leaderless
  }

  consensusType(): ConsensusType {
    return ConsensusType.Gossip;
  }
}

// -- ConsensusManager -------------------------------------------------------

/**
 * Manages multiple consensus layers, routing proposals to the correct layer.
 * Port of Rust ConsensusManager.
 */
export class ConsensusManager {
  private layers: Map<ConsensusType, ConsensusLayer> = new Map();

  /** Register a consensus layer. */
  register(layer: ConsensusLayer): void {
    this.layers.set(layer.consensusType(), layer);
  }

  /** Propose a value to the specified consensus type. */
  async propose(consensusType: ConsensusType, value: Uint8Array): Promise<boolean> {
    const layer = this.layers.get(consensusType);
    if (!layer) {
      throw consensusFailed(`no layer for ${consensusType}`);
    }
    return layer.propose(value);
  }

  /**
   * Determine which consensus type to use for a given syscall type.
   * Mirrors Rust submit_for_syscall routing logic.
   */
  submitForSyscall(syscallType: string): ConsensusType {
    switch (syscallType) {
      case 'StateMutate':
        return ConsensusType.Pbft;
      case 'ClusterMembership':
      case 'ConfigChange':
      case 'TokenRevocation':
        return ConsensusType.Raft;
      default:
        return ConsensusType.Gossip;
    }
  }

  /** Get the leader for a specific consensus type. */
  leader(consensusType: ConsensusType): NodeId | undefined {
    return this.layers.get(consensusType)?.currentLeader();
  }

  /** Get a registered layer by type. */
  getLayer(consensusType: ConsensusType): ConsensusLayer | undefined {
    return this.layers.get(consensusType);
  }
}

// -- Helpers ----------------------------------------------------------------

function sha256(input: string): Uint8Array {
  const hash = createHash('sha256').update(input).digest();
  return new Uint8Array(hash);
}

function buffersEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}
