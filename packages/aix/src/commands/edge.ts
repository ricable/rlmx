// ---------------------------------------------------------------------------
// aix edge — Edge inference commands
// ---------------------------------------------------------------------------

import { Command } from 'commander';
import chalk from 'chalk';

export function registerEdgeCommand(program: Command): void {
  const edge = program
    .command('edge')
    .description('Edge inference management');

  edge
    .command('status')
    .description('Show edge inference status')
    .action(async () => {
      console.log(chalk.cyan('Edge Inference Status:'));
      console.log(`  ${chalk.white('engine:')}  ${chalk.yellow('unavailable')}`);
      console.log(`  ${chalk.white('model:')}   none loaded`);
      console.log(`  ${chalk.white('backend:')} candle (stub)`);
      console.log();
      console.log(chalk.gray('  Set RLMX_EDGE_MODEL to enable edge inference.'));
    });

  edge
    .command('models')
    .description('List available edge models')
    .action(async () => {
      console.log(chalk.cyan('Available Edge Models:'));
      console.log(chalk.yellow('  No models found in model directory.'));
      console.log(chalk.gray('  Default path: ~/.rlmx/models'));
    });
}
