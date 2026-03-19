// @aix/deploy — Universal Agent Deployment Layer

export type {
  AgentManifest,
  AgentOrigin,
  Modality,
  Capability,
  ResourceEnvelope,
  TransportSpec,
  TransportType,
  BridgeConfig,
  BridgeRuntime,
  DiscoverySpec,
  SecuritySpec,
  SensorSpec,
  LearningSpec,
  DeploymentSpec,
  DeploymentProfile,
} from './manifest.js';

export { ALL_MODALITIES, TRANSPORT_TYPES, createManifest } from './manifest.js';

export { DomainMapper } from './domain-mapper.js';

export { ManifestRegistry } from './registry.js';
export type { ManifestSearchOptions, OriginType } from './registry.js';

export { validateManifest } from './validation.js';
export type { ValidationError, ValidationResult } from './validation.js';

export {
  DeployError,
  DeployErrorCode,
  manifestValidation,
  manifestNotFound,
  duplicateManifest,
  transportUnavailable,
  discoveryFailed,
  seedBridgeError,
  profileNotFound,
  templateNotFound,
  dimensionMismatch,
  generatorError,
} from './error.js';

// Transport
export type { TransportAdapter, AgentRequest, AgentResponse } from './transport.js';
export {
  McpTransportAdapter,
  RestTransportAdapter,
  WebSocketTransportAdapter,
  StubTransportAdapter,
  createTransportAdapter,
} from './transport.js';

// Bridge adapters (ADR-033)
export { ClaudeCodeBridgeAdapter } from './bridge-claude-code.js';
export { CodexBridgeAdapter } from './bridge-codex.js';
export { HttpGenericBridgeAdapter } from './bridge-http-generic.js';

// Discovery Bridge
export { DiscoveryBridge } from './bridge.js';
export type { DiscoveredNode } from './bridge.js';

// Seed Bridge
export { SeedBridge } from './seed-bridge.js';
export type {
  SeedBridgeConfig,
  SeedStatus,
  SensorReading,
  VectorResult,
  WitnessEntry,
  WitnessVerification,
  ClusterInfo,
  PeerInfo,
} from './seed-bridge.js';

// Profiles
export { DEPLOYMENT_PROFILES, PROFILE_NAMES, getProfile, getProfileOrThrow, listProfiles, fitsProfile } from './profiles.js';

// Templates
export { getTemplates, getTemplate, getTemplatesByDomain, getTemplatesByOrigin, getTemplatesByModality, TEMPLATE_COUNT } from './templates.js';

// Generator
export { generateSystemdUnit, generateFleetManifest, generateVerificationScript, generateDockerCompose } from './generator.js';
export type { DeployFleetManifest } from './generator.js';
