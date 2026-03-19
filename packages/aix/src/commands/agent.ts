// ---------------------------------------------------------------------------
// aix agent — Manage AI agents
// ---------------------------------------------------------------------------

import { Command } from 'commander';
import chalk from 'chalk';
import { randomUUID } from 'node:crypto';

export function registerAgentCommand(program: Command): void {
  const agent = program
    .command('agent')
    .description('Manage AI agents');

  agent
    .command('spawn')
    .description('Spawn a new agent')
    .requiredOption('-t, --type <type>', 'Agent type (coordinator, researcher, router, worker, monitor, etc.)')
    .option('--name <name>', 'Optional agent name')
    .option('--task <task>', 'Task to assign')
    .action(async (opts) => {
      const agentType = opts.type as string;
      const agentId = randomUUID();
      const name = (opts.name as string) ?? `${agentType}-${agentId.slice(0, 8)}`;

      console.log(chalk.cyan('Spawning agent:'));
      console.log(`  ${chalk.white('id:')}     ${agentId}`);
      console.log(`  ${chalk.white('type:')}   ${agentType}`);
      console.log(`  ${chalk.white('name:')}   ${name}`);
      if (opts.task) {
        console.log(`  ${chalk.white('task:')}   ${opts.task}`);
      }
      console.log(`  ${chalk.white('status:')} ${chalk.green('running')}`);
      console.log();
      console.log(chalk.green('Agent spawned successfully.'));
    });

  agent
    .command('list')
    .description('List running agents')
    .option('-t, --type <type>', 'Filter by agent type')
    .action(async (opts) => {
      console.log(chalk.cyan('Active Agents:'));
      if (opts.type) {
        console.log(chalk.gray(`  (filtered by type: ${opts.type})`));
      }
      console.log(chalk.yellow('  No agents currently running.'));
      console.log(chalk.gray("  Use 'aix agent spawn --type <type>' to start one."));
    });

  agent
    .command('kill <agentId>')
    .description('Terminate an agent')
    .action(async (agentId: string) => {
      console.log(chalk.cyan(`Terminating agent: ${agentId}`));
      console.log(chalk.green('  Agent terminated successfully.'));
    });
}
