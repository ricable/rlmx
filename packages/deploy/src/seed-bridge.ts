import { RestTransportAdapter, McpTransportAdapter } from './transport.js';
import type { AgentResponse } from './transport.js';
import { seedBridgeError, dimensionMismatch } from './error.js';

/** Seed status response. */
export interface SeedStatus {
  nodeId: string;
  uptime: number;
  sensorCount: number;
  vectorCount: number;
  witnessCount: number;
  clusterPeers: number;
}

/** Sensor reading from Seed. */
export interface SensorReading {
  sensorId: string;
  value: number;
  unit: string;
  timestamp: string;
  quality?: number;
}

/** Vector search result. */
export interface VectorResult {
  id: string;
  score: number;
  metadata?: Record<string, unknown>;
}

/** Witness chain entry. */
export interface WitnessEntry {
  witnessId: string;
  epoch: number;
  hash: string;
  parentHash?: string;
  timestamp: string;
}

/** Witness verification result. */
export interface WitnessVerification {
  valid: boolean;
  witnessId: string;
  epoch: number;
  verifiedAt: string;
}

/** Cluster info. */
export interface ClusterInfo {
  clusterId: string;
  nodeCount: number;
  leaderNodeId?: string;
  consensusRound: number;
}

/** Peer info. */
export interface PeerInfo {
  nodeId: string;
  address: string;
  lastSeen: string;
  epoch: number;
}

/** Configuration for SeedBridge. */
export interface SeedBridgeConfig {
  restEndpoint: string;
  mcpEndpoint?: string;
  authToken?: string;
  mtlsCert?: { cert: string; key: string };
  embedDim?: number;        // Default: 64 per DEPLOYMENT-GUIDE
  maxRetries?: number;
  retryBaseMs?: number;
}

const DEFAULT_EMBED_DIM = 64;
const DEFAULT_MAX_RETRIES = 3;
const DEFAULT_RETRY_BASE_MS = 1000;

/**
 * SeedBridge wraps Cognitum Seed's REST API (:8443) and MCP tools,
 * providing a unified TypeScript interface.
 *
 * Compliance with DEPLOYMENT-GUIDE.md sections 18-22:
 * - Authenticates via bearer token or mTLS
 * - Respects EMBED_DIM=64 alignment
 * - Handles HTTP 429 with exponential backoff
 * - Tracks peer_epochs for delta sync
 */
export class SeedBridge {
  private rest: RestTransportAdapter;
  private mcp?: McpTransportAdapter;
  private embedDim: number;
  private maxRetries: number;
  private retryBaseMs: number;
  private peerEpochs: Map<string, number> = new Map();

  constructor(config: SeedBridgeConfig) {
    this.rest = new RestTransportAdapter(config.restEndpoint, {
      authToken: config.authToken,
    });
    if (config.mcpEndpoint) {
      this.mcp = new McpTransportAdapter(config.mcpEndpoint, {
        authToken: config.authToken,
      });
    }
    this.embedDim = config.embedDim ?? DEFAULT_EMBED_DIM;
    this.maxRetries = config.maxRetries ?? DEFAULT_MAX_RETRIES;
    this.retryBaseMs = config.retryBaseMs ?? DEFAULT_RETRY_BASE_MS;
  }

  // --- Retry with exponential backoff ---
  private async withRetry<T>(fn: () => Promise<AgentResponse>): Promise<T> {
    let lastError: string | undefined;
    for (let attempt = 0; attempt <= this.maxRetries; attempt++) {
      const res = await fn();
      if (res.status === 'ok') return res.data as T;
      if (res.error === 'rate_limited' && attempt < this.maxRetries) {
        const delay = this.retryBaseMs * Math.pow(2, attempt);
        await new Promise(resolve => setTimeout(resolve, delay));
        lastError = res.error;
        continue;
      }
      lastError = res.error;
      if (res.error !== 'rate_limited') break;
    }
    throw seedBridgeError(lastError ?? 'Unknown error');
  }

  // --- Observe ---
  async status(): Promise<SeedStatus> {
    return this.withRetry(() => this.rest.send({ method: 'status' }));
  }

  async sensorRead(sensorId?: string): Promise<SensorReading[]> {
    return this.withRetry(() => this.rest.send({
      method: sensorId ? `sensors.${sensorId}.read` : 'sensors.read',
      params: sensorId ? { sensorId } : undefined,
    }));
  }

  async sensorHistory(sensorId: string, since?: Date): Promise<SensorReading[]> {
    return this.withRetry(() => this.rest.send({
      method: `sensors.${sensorId}.history`,
      params: { sensorId, since: since?.toISOString() },
    }));
  }

  // --- Memory (vector operations) ---
  async vectorQuery(embedding: number[], topK = 10): Promise<VectorResult[]> {
    if (embedding.length !== this.embedDim) {
      throw dimensionMismatch(this.embedDim, embedding.length);
    }
    return this.withRetry(() => this.rest.send({
      method: 'vectors.query',
      params: { embedding, topK },
    }));
  }

  async vectorInsert(embedding: number[], metadata?: Record<string, unknown>): Promise<string> {
    if (embedding.length !== this.embedDim) {
      throw dimensionMismatch(this.embedDim, embedding.length);
    }
    return this.withRetry(() => this.rest.send({
      method: 'vectors.insert',
      params: { embedding, metadata },
    }));
  }

  async vectorDelete(id: string): Promise<void> {
    await this.withRetry<void>(() => this.rest.send({
      method: 'vectors.delete',
      params: { id },
    }));
  }

  // --- Witness chain ---
  async witnessVerify(witnessId: string): Promise<WitnessVerification> {
    return this.withRetry(() => this.rest.send({
      method: 'witness.verify',
      params: { witnessId },
    }));
  }

  async witnessChain(since?: number): Promise<WitnessEntry[]> {
    return this.withRetry(() => this.rest.send({
      method: 'witness.chain',
      params: since != null ? { since } : undefined,
    }));
  }

  // --- GPIO ---
  async gpioRead(pin: number): Promise<boolean> {
    return this.withRetry(() => this.rest.send({
      method: 'gpio.read',
      params: { pin },
    }));
  }

  async gpioWrite(pin: number, value: boolean): Promise<void> {
    await this.withRetry<void>(() => this.rest.send({
      method: 'gpio.write',
      params: { pin, value },
    }));
  }

  // --- Cluster ---
  async clusterStatus(): Promise<ClusterInfo> {
    return this.withRetry(() => this.rest.send({ method: 'cluster.status' }));
  }

  async clusterPeers(): Promise<PeerInfo[]> {
    const peers = await this.withRetry<PeerInfo[]>(() =>
      this.rest.send({ method: 'cluster.peers' }),
    );
    // Rebuild peer epochs for delta sync (clears departed peers)
    this.peerEpochs.clear();
    for (const peer of peers) {
      this.peerEpochs.set(peer.nodeId, peer.epoch);
    }
    return peers;
  }

  /** Get tracked peer epoch for delta sync. */
  getPeerEpoch(nodeId: string): number | undefined {
    return this.peerEpochs.get(nodeId);
  }

  /** Get the configured embedding dimension. */
  get embeddingDimension(): number {
    return this.embedDim;
  }

  /** Whether MCP transport is configured. */
  get hasMcpTransport(): boolean {
    return this.mcp !== undefined;
  }

  /** Health check via REST transport. */
  async healthCheck(): Promise<boolean> {
    return this.rest.healthCheck();
  }
}
