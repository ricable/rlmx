// ---------------------------------------------------------------------------
// aix — CLI entry point
//
// Ports the Rust rlmx-cli to TypeScript using commander.js.
// All 14 command groups from the Rust CLI are registered:
//   serve, query, voice, agent, swarm, marketplace, mesh, billing,
//   federation, engagement, phone, research, sandbox, edge
// ---------------------------------------------------------------------------

import { Command } from 'commander';
import { registerServeCommand } from './commands/serve.js';
import { registerQueryCommand } from './commands/query.js';
import { registerVoiceCommand } from './commands/voice.js';
import { registerAgentCommand } from './commands/agent.js';
import { registerSwarmCommand } from './commands/swarm.js';
import { registerMarketplaceCommand } from './commands/marketplace.js';
import { registerMeshCommand } from './commands/mesh.js';
import { registerBillingCommand } from './commands/billing.js';
import { registerFederationCommand } from './commands/federation.js';
import { registerEngagementCommand } from './commands/engagement.js';
import { registerPhoneCommand } from './commands/phone.js';
import { registerResearchCommand } from './commands/research.js';
import { registerSandboxCommand } from './commands/sandbox.js';
import { registerEdgeCommand } from './commands/edge.js';

/**
 * Build the CLI program with all command groups registered.
 * Exported for testing and programmatic use.
 */
export function createProgram(): Command {
  const program = new Command();

  program
    .name('aix')
    .version('0.1.0')
    .description('AIX cognition kernel command-line interface');

  registerServeCommand(program);
  registerQueryCommand(program);
  registerVoiceCommand(program);
  registerAgentCommand(program);
  registerSwarmCommand(program);
  registerMarketplaceCommand(program);
  registerMeshCommand(program);
  registerBillingCommand(program);
  registerFederationCommand(program);
  registerEngagementCommand(program);
  registerPhoneCommand(program);
  registerResearchCommand(program);
  registerSandboxCommand(program);
  registerEdgeCommand(program);

  return program;
}

// Re-export command registrations for consumers
export { registerServeCommand } from './commands/serve.js';
export { registerQueryCommand } from './commands/query.js';
export { registerVoiceCommand } from './commands/voice.js';
export { registerAgentCommand } from './commands/agent.js';
export { registerSwarmCommand } from './commands/swarm.js';
export { registerMarketplaceCommand } from './commands/marketplace.js';
export { registerMeshCommand } from './commands/mesh.js';
export { registerBillingCommand } from './commands/billing.js';
export { registerFederationCommand } from './commands/federation.js';
export { registerEngagementCommand } from './commands/engagement.js';
export { registerPhoneCommand } from './commands/phone.js';
export { registerResearchCommand } from './commands/research.js';
export { registerSandboxCommand } from './commands/sandbox.js';
export { registerEdgeCommand } from './commands/edge.js';

// ---------------------------------------------------------------------------
// Run when invoked as entry point (bin/cli.js -> dist/index.js)
// Guard: only auto-parse when run directly, not when imported for testing.
// ---------------------------------------------------------------------------

const isDirectRun =
  typeof process !== 'undefined' &&
  process.argv[1] &&
  (process.argv[1].endsWith('/cli.js') || process.argv[1].endsWith('/index.js'));

if (isDirectRun) {
  const program = createProgram();
  program.parseAsync(process.argv).catch((err: Error) => {
    console.error(`error: ${err.message}`);
    process.exit(1);
  });
}
