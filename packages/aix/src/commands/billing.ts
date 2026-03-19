// ---------------------------------------------------------------------------
// aix billing — Subscription billing and usage (ADR-025)
// ---------------------------------------------------------------------------

import { Command } from 'commander';
import chalk from 'chalk';

export function registerBillingCommand(program: Command): void {
  const billing = program
    .command('billing')
    .description('Subscription billing and usage');

  billing
    .command('status')
    .description('Show subscription tier and usage')
    .action(async () => {
      console.log(chalk.cyan('Billing Status:'));
      console.log(`  ${chalk.white('tier:')}         ${chalk.green('Free')}`);
      console.log(`  ${chalk.white('status:')}       active`);
      console.log(`  ${chalk.white('agent_limit:')}  5`);
      console.log(`  ${chalk.white('agents_used:')}  0`);
      console.log(`  ${chalk.white('period:')}       current`);
    });

  billing
    .command('upgrade')
    .description('Upgrade subscription tier')
    .requiredOption('--tier <tier>', 'Target tier (free, personal, family, pro, enterprise, developer)')
    .action(async (opts) => {
      const tier = opts.tier as string;
      console.log(chalk.cyan(`Upgrading to tier: ${tier}`));
      console.log(chalk.yellow('  Billing upgrade is in stub mode.'));
    });

  billing
    .command('usage')
    .description('Show current period usage')
    .action(async () => {
      console.log(chalk.cyan('Usage Metrics:'));
      console.log(`  ${chalk.white('inference_tokens:')}   0`);
      console.log(`  ${chalk.white('cloud_burst_tokens:')} 0`);
      console.log(`  ${chalk.white('agent_hours:')}        0`);
    });

  billing
    .command('family')
    .description('Manage family group')
    .action(async () => {
      console.log(chalk.cyan('Family Group:'));
      console.log(chalk.yellow('  No family group configured.'));
      console.log(`  ${chalk.white('max_members:')} 6`);
    });

  billing
    .command('developer')
    .description('Developer account info')
    .action(async () => {
      console.log(chalk.cyan('Developer Account:'));
      console.log(`  ${chalk.white('revenue_split:')} 70/30 (developer/platform)`);
      console.log(`  ${chalk.white('payout_threshold:')} $50`);
      console.log(`  ${chalk.white('balance:')} $0.00`);
    });
}
