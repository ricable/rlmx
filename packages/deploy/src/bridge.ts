import type { AgentManifest } from './manifest.js';
import { LifeDomain } from '@aix/shared';

/** A node discovered via mDNS or other discovery mechanism. */
export interface DiscoveredNode {
  id: string;
  name: string;
  host: string;
  port: number;
  serviceType: string;
  origin: string;         // 'seed' | 'rlmx' | 'custom'
  txtRecords: Record<string, string>;
  discoveredAt: string;   // ISO 8601
}

type NodeEventHandler = (node: DiscoveredNode) => void;

const DEFAULT_SERVICE_TYPES: Record<string, string> = {
  '_cognitum._tcp': 'seed',
  '_rlmx._tcp': 'rlmx',
};

/**
 * DiscoveryBridge provides unified mDNS-based discovery across
 * Cognitum Seed nodes, RLMX mesh nodes, and custom services.
 *
 * In production this would use a mDNS library (e.g. multicast-dns npm).
 * This implementation provides the domain model and event interface.
 */
export class DiscoveryBridge {
  private serviceTypes: Map<string, string>;
  private nodes: Map<string, DiscoveredNode> = new Map();
  private handlers: { discovered: NodeEventHandler[]; lost: NodeEventHandler[] } = {
    discovered: [],
    lost: [],
  };
  private running = false;

  constructor(options?: { customServiceTypes?: Record<string, string> }) {
    this.serviceTypes = new Map([
      ...Object.entries(DEFAULT_SERVICE_TYPES),
      ...Object.entries(options?.customServiceTypes ?? {}),
    ]);
  }

  /** Start listening for mDNS announcements. */
  async start(): Promise<void> {
    this.running = true;
    // In production: bind mDNS listener on port 5353
    // This is the domain model skeleton; transport is injected at integration time
  }

  /** Stop listening and clean up all state. */
  async stop(): Promise<void> {
    this.running = false;
    this.nodes.clear();
    this.handlers.discovered = [];
    this.handlers.lost = [];
  }

  /** Whether the bridge is currently running. */
  get isRunning(): boolean {
    return this.running;
  }

  /** Get all discovered nodes. */
  getNodes(): DiscoveredNode[] {
    return [...this.nodes.values()];
  }

  /** Get nodes filtered by origin. */
  getNodesByOrigin(origin: string): DiscoveredNode[] {
    const result: DiscoveredNode[] = [];
    for (const n of this.nodes.values()) {
      if (n.origin === origin) result.push(n);
    }
    return result;
  }

  /** Register an event handler. */
  on(event: 'node-discovered' | 'node-lost', handler: NodeEventHandler): void {
    if (event === 'node-discovered') this.handlers.discovered.push(handler);
    else this.handlers.lost.push(handler);
  }

  /** Remove an event handler. */
  off(event: 'node-discovered' | 'node-lost', handler: NodeEventHandler): void {
    if (event === 'node-discovered') {
      this.handlers.discovered = this.handlers.discovered.filter(h => h !== handler);
    } else {
      this.handlers.lost = this.handlers.lost.filter(h => h !== handler);
    }
  }

  /**
   * Inject a discovered node (called by mDNS transport layer or tests).
   * Emits 'node-discovered' event if new.
   */
  addNode(node: DiscoveredNode): void {
    const isNew = !this.nodes.has(node.id);
    this.nodes.set(node.id, node);
    if (isNew) {
      for (const handler of this.handlers.discovered) handler(node);
    }
  }

  /** Remove a node. Emits 'node-lost' event. */
  removeNode(nodeId: string): void {
    const node = this.nodes.get(nodeId);
    if (node) {
      this.nodes.delete(nodeId);
      for (const handler of this.handlers.lost) handler(node);
    }
  }

  /** Get the origin label for a service type. */
  resolveServiceType(serviceType: string): string | undefined {
    return this.serviceTypes.get(serviceType);
  }

  /** Register a custom service type mapping. */
  registerServiceType(serviceType: string, origin: string): void {
    this.serviceTypes.set(serviceType, origin);
  }

  /**
   * Auto-generate an AgentManifest from a discovered Seed node.
   * Uses TXT records for sensor profile, capabilities, etc.
   */
  manifestForSeedNode(node: DiscoveredNode): AgentManifest {
    const sensorProfile = node.txtRecords['sensor_profile'] ?? 'default';
    const name = node.txtRecords['agent_name'] ?? node.name;

    return {
      id: `seed-${node.id}`,
      name,
      version: node.txtRecords['version'] ?? '0.1.0',
      origin: {
        type: 'seed',
        mcpEndpoint: `http://${node.host}:${node.port}/mcp`,
        sensorProfile,
        restEndpoint: `https://${node.host}:8443`,
      },
      lifeDomain: LifeDomain.Home,
      domainTags: ['sensor', 'iot', sensorProfile],
      capabilities: [{ name: 'observe', required: true }],
      modalities: ['sensor'],
      resourceEnvelope: {
        cpuCores: 1,
        memoryMb: 256,
        diskMb: 512,
        maxRuntimeMs: 0,   // persistent
      },
      transports: [
        { type: 'mcp', endpoint: `http://${node.host}:${node.port}/mcp` },
        { type: 'rest', baseUrl: `https://${node.host}:8443` },
      ],
      discovery: {
        serviceType: '_cognitum._tcp',
        port: node.port,
        txtRecords: node.txtRecords,
      },
      security: { authMethod: 'bearer' },
      deployment: { profiles: ['seed-compatible'] },
      sensorConfig: {
        interfaces: ['gpio', 'i2c'],
        driftDetection: true,
        samplingIntervalMs: 1000,
      },
      metadata: { discoveredAt: node.discoveredAt, host: node.host },
    };
  }
}
