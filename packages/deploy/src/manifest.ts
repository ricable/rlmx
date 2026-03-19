import type { LifeDomain, AgentType, SyscallPermission } from '@aix/shared';

// --- Modality ---

export type Modality = 'text' | 'voice' | 'vision' | 'sensor' | 'haptic' | 'multimodal';

export const ALL_MODALITIES: readonly Modality[] = [
  'text', 'voice', 'vision', 'sensor', 'haptic', 'multimodal',
] as const;

// --- Capability ---

export interface Capability {
  name: string;
  description?: string;
  required: boolean;
}

// --- Resource Envelope ---

export interface ResourceEnvelope {
  cpuCores: number;
  memoryMb: number;
  gpuType?: 'none' | 'metal' | 'cuda' | 'webgpu';
  diskMb: number;
  maxRuntimeMs: number;
}

// --- Transport specs ---

export interface BridgeConfig {
  command?: string;
  endpoint?: string;
  model?: string;
  authEnvVar?: string;
}

export type BridgeRuntime = 'claude-code' | 'codex' | 'cursor' | 'opencode' | 'http-generic';

export type TransportSpec =
  | { type: 'mcp'; endpoint: string }
  | { type: 'rest'; baseUrl: string }
  | { type: 'ws'; url: string }
  | { type: 'quic'; addr: string }
  | { type: 'mqtt'; broker: string; topic: string }
  | { type: 'grpc'; endpoint: string }
  | { type: 'broadcast-channel'; channel: string }
  | { type: 'bridge'; runtime: BridgeRuntime; config: BridgeConfig };

export const TRANSPORT_TYPES = [
  'mcp', 'rest', 'ws', 'quic', 'mqtt', 'grpc', 'broadcast-channel', 'bridge',
] as const;

export type TransportType = (typeof TRANSPORT_TYPES)[number];

// --- Discovery spec ---

export interface DiscoverySpec {
  /** Service type for DNS-SD / mDNS, e.g. '_cognitum._tcp', '_rlmx._tcp' */
  serviceType: string;
  dnsSd?: boolean;
  port?: number;
  txtRecords?: Record<string, string>;
}

// --- Security spec ---

export interface SecuritySpec {
  authMethod: 'bearer' | 'mtls' | 'macaroon' | 'none';
  attestation?: 'ed25519' | 'hmac' | 'none';
  sandboxProfile?: string;
  rateLimit?: { maxRequestsPerMinute: number };
}

// --- Sensor spec ---

export interface SensorSpec {
  interfaces: ('gpio' | 'i2c' | 'spi' | 'uart' | 'usb')[];
  driftDetection?: boolean;
  samplingIntervalMs?: number;
  calibrationRequired?: boolean;
}

// --- Learning spec ---

export interface LearningSpec {
  sona?: boolean;
  federation?: boolean;
  driftDetectors?: string[];
  ewcLambda?: number;
}

// --- Deployment spec ---

export interface DeploymentSpec {
  profiles: string[];
  systemdUnit?: string;
  containerImage?: string;
  rvfPath?: string;
}

// --- Agent Origin ---

export type AgentOrigin =
  | { type: 'rlmx'; agentType: AgentType; permissions: SyscallPermission[] }
  | { type: 'seed'; mcpEndpoint: string; sensorProfile: string; restEndpoint?: string }
  | { type: 'external'; protocol: string; endpoint: string }
  | { type: 'custom'; handler: string };

// --- Agent Manifest ---

export interface AgentManifest {
  id: string;
  name: string;
  version: string;
  origin: AgentOrigin;
  lifeDomain: LifeDomain;
  domainTags: string[];
  capabilities: Capability[];
  modalities: Modality[];
  resourceEnvelope: ResourceEnvelope;
  transports: TransportSpec[];
  discovery?: DiscoverySpec;
  security: SecuritySpec;
  deployment: DeploymentSpec;
  sensorConfig?: SensorSpec;
  learning?: LearningSpec;
  metadata: Record<string, unknown>;
}

// --- Deployment Profile ---

export interface DeploymentProfile {
  name: string;
  target: string;
  zone: string;
  features: string[];
  maxPowerWatts?: number;
  maxMemoryMb?: number;
}

// --- Helpers ---

export function createManifest(
  partial: Omit<AgentManifest, 'metadata'> & { metadata?: Record<string, unknown> },
): AgentManifest {
  return { ...partial, metadata: partial.metadata ?? {} };
}
