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

/** Safe ID pattern for use in shell commands and file names. */
const SAFE_ID_RE = /^[a-z0-9][a-z0-9._-]*$/;

/** Escape a string for safe YAML double-quoted value. */
function yamlEscape(s: string): string {
  return s.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
}

/**
 * Generate a systemd unit file for a manifest + profile.
 */
export function generateSystemdUnit(manifest: AgentManifest, profile: DeploymentProfile): string {
  const description = `${manifest.name} (${manifest.id}) — ${profile.name}`;
  const safeId = SAFE_ID_RE.test(manifest.id) ? manifest.id : manifest.id.replace(/[^a-z0-9._-]/g, '-');
  const execStart = manifest.origin.type === 'seed'
    ? `/usr/local/bin/cognitum-seed --profile ${safeId}`
    : `/usr/local/bin/aix deploy spawn --manifest ${safeId}`;

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
export function generateVerificationScript(seedDevices: AgentManifest[]): string {
  const checks = seedDevices
    .filter(m => m.origin.type === 'seed')
    .map(m => {
      const origin = m.origin as { type: 'seed'; restEndpoint?: string; mcpEndpoint: string };
      const restUrl = origin.restEndpoint ?? 'http://localhost:8443';
      const name = m.name;
      const id = m.id;
      return `# --- ${name} (${id}) ---
echo "Checking ${name}..."
curl -sf "${restUrl}/health" || echo "FAIL: ${name} health check"
curl -sf "${restUrl}/sensors/read" || echo "FAIL: ${name} sensor read"
curl -sf "${restUrl}/witness/chain" || echo "FAIL: ${name} witness chain"
echo "${name}: OK"
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
  const lines = ['version: "3.8"', 'services:'];

  for (const m of manifests) {
    const serviceName = m.id.replace(/[^a-z0-9-]/g, '-');
    const image = m.deployment.containerImage ?? `aix/${m.id}:${m.version}`;

    lines.push(`  ${serviceName}:`);
    lines.push(`    image: "${yamlEscape(image)}"`);
    lines.push(`    restart: "unless-stopped"`);
    lines.push(`    environment:`);
    lines.push(`      AGENT_ID: "${yamlEscape(m.id)}"`);
    lines.push(`      AGENT_NAME: "${yamlEscape(m.name)}"`);
    lines.push(`      NODE_ENV: "production"`);
    lines.push(`    deploy:`);
    lines.push(`      resources:`);
    lines.push(`        limits:`);
    lines.push(`          memory: "${m.resourceEnvelope.memoryMb}M"`);
    lines.push(`          cpus: "${m.resourceEnvelope.cpuCores}"`);
  }

  return lines.join('\n') + '\n';
}
