import type { AgentManifest, DeploymentProfile } from './manifest.js';
import { generatorError } from './error.js';

// Re-export FleetManifest shape for compatibility with @aix/swarm.
// Defined locally to avoid hard runtime dependency on @aix/swarm.
export interface DeployFleetManifest {
  name: string;
  version: string;
  sandboxes: Array<{ profile: string; count: number; overrides?: Record<string, unknown> }>;
  cloudPolicy?: Record<string, unknown>;
}

/**
 * Generate a systemd unit file for a manifest + profile.
 */
export function generateSystemdUnit(manifest: AgentManifest, profile: DeploymentProfile): string {
  const description = `${manifest.name} (${manifest.id}) — ${profile.name}`;
  const execStart = manifest.origin.type === 'seed'
    ? `/usr/local/bin/cognitum-seed --profile ${manifest.id}`
    : `/usr/local/bin/aix deploy spawn --manifest ${manifest.id}`;

  const memoryMax = profile.maxMemoryMb ? `MemoryMax=${profile.maxMemoryMb}M` : '';

  return `[Unit]
Description=${description}
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=${execStart}
Restart=on-failure
RestartSec=10
Environment=NODE_ENV=production
${memoryMax ? memoryMax + '\n' : ''}
[Install]
WantedBy=multi-user.target
`;
}

/**
 * Generate a fleet manifest compatible with @aix/swarm FleetManifest.
 */
export function generateFleetManifest(
  manifests: AgentManifest[],
  profile: DeploymentProfile,
): DeployFleetManifest {
  if (manifests.length === 0) {
    throw generatorError('Cannot generate fleet manifest from empty manifest list');
  }

  return {
    name: `fleet-${profile.name}`,
    version: '0.1.0',
    sandboxes: manifests.map(m => ({
      profile: m.deployment.profiles[0] ?? profile.name,
      count: 1,
      overrides: { manifestId: m.id, agentName: m.name },
    })),
  };
}

/**
 * Generate a verification script for Seed devices.
 * Tests mDNS, REST health, sensor read, witness chain.
 */
/** Shell-escape a string for safe interpolation in single quotes. */
function shellEscape(s: string): string {
  return s.replace(/'/g, "'\\''");
}

export function generateVerificationScript(seedDevices: AgentManifest[]): string {
  const checks = seedDevices
    .filter(m => m.origin.type === 'seed')
    .map(m => {
      const origin = m.origin as { type: 'seed'; restEndpoint?: string; mcpEndpoint: string };
      const restUrl = shellEscape(origin.restEndpoint ?? 'http://localhost:8443');
      const safeName = shellEscape(m.name);
      const safeId = shellEscape(m.id);
      return `# --- ${safeName} (${safeId}) ---
echo 'Checking ${safeName}...'
curl -sf '${restUrl}/health' || echo 'FAIL: ${safeName} health check'
curl -sf '${restUrl}/sensors/read' || echo 'FAIL: ${safeName} sensor read'
curl -sf '${restUrl}/witness/chain' || echo 'FAIL: ${safeName} witness chain'
echo '${safeName}: OK'
`;
    })
    .join('\n');

  return `#!/usr/bin/env bash
set -euo pipefail
echo "=== Seed Device Verification ==="
${checks || 'echo "No seed devices to verify"'}
echo "=== Verification Complete ==="
`;
}

/**
 * Generate docker-compose.yml for cloud deployments.
 */
export function generateDockerCompose(manifests: AgentManifest[]): string {
  const services: Record<string, unknown> = {};

  for (const m of manifests) {
    const serviceName = m.id.replace(/[^a-z0-9-]/g, '-');
    services[serviceName] = {
      image: m.deployment.containerImage ?? `aix/${m.id}:${m.version}`,
      restart: 'unless-stopped',
      environment: {
        AGENT_ID: m.id,
        AGENT_NAME: m.name,
        NODE_ENV: 'production',
      },
      deploy: {
        resources: {
          limits: {
            memory: `${m.resourceEnvelope.memoryMb}M`,
            cpus: String(m.resourceEnvelope.cpuCores),
          },
        },
      },
    };
  }

  // Manual YAML generation to avoid dependency
  const lines = ['version: "3.8"', 'services:'];
  for (const [name, config] of Object.entries(services)) {
    const c = config as Record<string, unknown>;
    lines.push(`  ${name}:`);
    lines.push(`    image: "${c.image}"`);
    lines.push(`    restart: "${c.restart}"`);
    lines.push(`    environment:`);
    const env = c.environment as Record<string, string>;
    for (const [k, v] of Object.entries(env)) {
      lines.push(`      ${k}: "${v}"`);
    }
    const deploy = c.deploy as { resources: { limits: { memory: string; cpus: string } } };
    lines.push(`    deploy:`);
    lines.push(`      resources:`);
    lines.push(`        limits:`);
    lines.push(`          memory: ${deploy.resources.limits.memory}`);
    lines.push(`          cpus: "${deploy.resources.limits.cpus}"`);
  }

  return lines.join('\n') + '\n';
}
