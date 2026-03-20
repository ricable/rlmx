/**
 * UC1 — Deployment profiles for voice-first swarm.
 *
 * Illustrative only — not built or tested by CI.
 */

interface DeployProfile {
  name: string;
  zone: string;
  inference: string;
  maxAgents: number;
  features: string[];
}

/** Deployment profiles for UC1 devices. */
export const uc1Profiles: DeployProfile[] = [
  {
    name: 'phone',
    zone: 'A-Mobile',
    inference: 'WASM + WebGPU',
    maxAgents: 50,
    features: [
      'on-device-stt',
      'micro-lora',
      'swarm-consensus',
      'voice-response',
      'haptic-feedback',
    ],
  },
  {
    name: 'laptop',
    zone: 'A-Desktop',
    inference: 'NAPI-RS native',
    maxAgents: 200,
    features: [
      'full-swarm',
      'researcher-agents',
      'visual-dashboard',
      'cross-device-sync',
    ],
  },
  {
    name: 'hub',
    zone: 'C-Hub',
    inference: 'Edge (TinyLlama Q4, 15 tok/s)',
    maxAgents: 100,
    features: [
      'privacy-anchor',
      'long-term-storage',
      'federation-endpoint',
      'home-iot-bridge',
    ],
  },
  {
    name: 'cloud',
    zone: 'D-Cloud',
    inference: 'Full models (SkyPilot GPU)',
    maxAgents: 1000,
    features: [
      'marketplace-burst',
      'heavy-inference',
      'federated-aggregation',
    ],
  },
];
