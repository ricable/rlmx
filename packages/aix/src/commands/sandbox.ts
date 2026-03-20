// ---------------------------------------------------------------------------
// aix sandbox — Manage sandbox environments (ADR-011)
// ---------------------------------------------------------------------------

import { Command } from 'commander';
import chalk from 'chalk';
import { randomUUID } from 'node:crypto';

export function registerSandboxCommand(program: Command): void {
  const sandbox = program
    .command('sandbox')
    .description('Manage sandbox environments');

  sandbox
    .command('spawn')
    .description('Spawn a sandbox from a profile')
    .requiredOption('--profile <profile>', 'Profile name to spawn')
    .option('--zone <zone>', 'Zone override')
    .action(async (opts) => {
      const profile = opts.profile as string;
      const sandboxId = randomUUID();
      console.log(chalk.cyan('Spawning sandbox:'));
      console.log(`  ${chalk.white('sandbox_id:')} ${sandboxId}`);
      console.log(`  ${chalk.white('profile:')}    ${profile}`);
      if (opts.zone) {
        console.log(`  ${chalk.white('zone:')}       ${opts.zone}`);
      }
      console.log(`  ${chalk.white('status:')}     ${chalk.green('running')}`);
    });

  sandbox
    .command('terminate <sandboxId>')
    .description('Terminate a sandbox instance')
    .action(async (sandboxId: string) => {
      console.log(chalk.cyan(`Terminating sandbox: ${sandboxId}`));
      console.log(chalk.green('  Sandbox terminated.'));
    });

  sandbox
    .command('status <sandboxId>')
    .description('Show sandbox instance status')
    .action(async (sandboxId: string) => {
      console.log(chalk.cyan(`Sandbox Status: ${sandboxId}`));
      console.log(chalk.yellow('  Not found.'));
    });

  sandbox
    .command('list')
    .description('List all sandbox instances')
    .option('--state <state>', 'Filter by state')
    .option('--profile <profile>', 'Filter by profile')
    .action(async () => {
      console.log(chalk.cyan('Sandbox Instances:'));
      console.log(chalk.yellow('  No active sandboxes.'));
    });
}
