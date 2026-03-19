import type { DeploymentProfile } from './manifest.js';
import { profileNotFound } from './error.js';

export const DEPLOYMENT_PROFILES: Record<string, DeploymentProfile> = {
  'seed-compatible': {
    name: 'seed-compatible',
    target: 'rpi-zero-2w',
    zone: 'CEdge',
    features: ['sensor', 'drift-detection', 'witness'],
    maxPowerWatts: 1.4,
    maxMemoryMb: 256,
  },
  'rlmx-edge': {
    name: 'rlmx-edge',
    target: 'rpi5',
    zone: 'CEdge',
    features: ['ruvllm', 'gguf-inference'],
    maxPowerWatts: 12,
    maxMemoryMb: 4096,
  },
  'rlmx-cloud': {
    name: 'rlmx-cloud',
    target: 'cloud-vm',
    zone: 'BCloud',
    features: ['full-features', 'large-models'],
    maxPowerWatts: 35,
    maxMemoryMb: 16384,
  },
  'hybrid': {
    name: 'hybrid',
    target: 'rpi5',
    zone: 'CEdge',
    features: ['rlmx', 'seed-bridge'],
    maxPowerWatts: 12,
    maxMemoryMb: 4096,
  },
  'minimal': {
    name: 'minimal',
    target: 'arm-generic',
    zone: 'CEdge',
    features: ['routing-only'],
    maxPowerWatts: 2,
    maxMemoryMb: 512,
  },
  'browser': {
    name: 'browser',
    target: 'browser',
    zone: 'DBrowser',
    features: ['wasm', 'webgpu'],
    maxMemoryMb: 256,
  },
  'mobile': {
    name: 'mobile',
    target: 'phone',
    zone: 'AMobile',
    features: ['react-native', 'free-agents'],
    maxMemoryMb: 1024,
  },
};

export const PROFILE_NAMES = Object.keys(DEPLOYMENT_PROFILES) as ReadonlyArray<string>;

export function getProfile(name: string): DeploymentProfile | undefined {
  return DEPLOYMENT_PROFILES[name];
}

export function getProfileOrThrow(name: string): DeploymentProfile {
  const p = DEPLOYMENT_PROFILES[name];
  if (!p) {
    throw profileNotFound(name);
  }
  return p;
}

export function listProfiles(): DeploymentProfile[] {
  return Object.values(DEPLOYMENT_PROFILES);
}

/** Check if a manifest's resource envelope fits within a profile's constraints. */
export function fitsProfile(
  envelope: { memoryMb: number },
  profile: DeploymentProfile,
): boolean {
  if (profile.maxMemoryMb && envelope.memoryMb > profile.maxMemoryMb) return false;
  return true;
}
