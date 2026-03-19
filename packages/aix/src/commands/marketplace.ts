// ---------------------------------------------------------------------------
// aix marketplace — Agent marketplace
// ---------------------------------------------------------------------------

import { Command } from 'commander';
import chalk from 'chalk';

export function registerMarketplaceCommand(program: Command): void {
  const marketplace = program
    .command('marketplace')
    .description('Agent marketplace');

  marketplace
    .command('search')
    .description('Search for agents by domain')
    .requiredOption('--domain <domain>', 'Domain to search (finance, health, legal, etc.)')
    .action(async (opts) => {
      const domain = opts.domain as string;
      console.log(chalk.cyan(`Marketplace Search: ${domain}`));
      console.log(chalk.yellow('  No agents found matching your criteria.'));
      console.log(chalk.gray('  The marketplace is in stub mode.'));
    });

  marketplace
    .command('install')
    .description('Install an agent from the marketplace')
    .requiredOption('--agent <name>', 'Agent name to install')
    .action(async (opts) => {
      const agent = opts.agent as string;
      console.log(chalk.cyan(`Installing agent: ${agent}`));
      console.log(chalk.yellow('  Marketplace install is in stub mode.'));
    });

  marketplace
    .command('list')
    .description('List installed agents')
    .action(async () => {
      console.log(chalk.cyan('Installed Agents:'));
      console.log(chalk.yellow('  No agents installed.'));
    });

  marketplace
    .command('featured')
    .description('Show featured agents')
    .action(async () => {
      console.log(chalk.cyan('Featured Agents:'));
      console.log(chalk.yellow('  No featured agents available (stub mode).'));
    });

  marketplace
    .command('publish')
    .description('Publish an agent to the marketplace')
    .requiredOption('--path <path>', 'Path to agent RVF package')
    .action(async (opts) => {
      const path = opts.path as string;
      console.log(chalk.cyan(`Publishing agent from: ${path}`));
      console.log(chalk.yellow('  Publish is in stub mode.'));
    });
}
