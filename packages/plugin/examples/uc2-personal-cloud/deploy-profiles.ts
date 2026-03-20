/**
 * UC2 — Device-centric deployment profiles for personal agent cloud.
 *
 * Illustrative only — not built or tested by CI.
 */

interface MeshDeployProfile {
  name: string;
  zone: string;
  runtime: string;
  meshRole: 'primary' | 'compute' | 'anchor' | 'burst';
  maxAgents: number;
  features: string[];
}

/** Deployment profiles for UC2 mesh devices. */
export const uc2Profiles: MeshDeployProfile[] = [
  {
    name: 'phone',
    zone: 'A-Mobile',
    runtime: 'WASM + WebGPU',
    meshRole: 'primary',
    maxAgents: 50,
    features: [
      'wasm-agents',
      'webgpu-inference',
      'offline-queue',
      'streak-tracking',
      'haptic-notifications',
    ],
  },
  {
    name: 'laptop',
    zone: 'A-Desktop',
    runtime: 'NAPI-RS native',
    meshRole: 'compute',
    maxAgents: 200,
    features: [
      'napi-bridge',
      'lora-training',
      'heavy-research',
      'broadcast-channel',
    ],
  },
  {
    name: 'hub',
    zone: 'C-Hub',
    runtime: 'Native (systemd)',
    meshRole: 'anchor',
    maxAgents: 100,
    features: [
      'privacy-anchor',
      'long-term-storage',
      'federation-endpoint',
      'mesh-coordinator',
      'home-iot-bridge',
      'edge-inference',
    ],
  },
  {
    name: 'cloud',
    zone: 'D-Cloud',
    runtime: 'SkyPilot GPU',
    meshRole: 'burst',
    maxAgents: 500,
    features: [
      'gpu-inference',
      'federated-aggregation',
      'marketplace-hosting',
    ],
  },
];
