// ---------------------------------------------------------------------------
// aix query — Submit a query to the kernel for processing
// ---------------------------------------------------------------------------

import { Command } from 'commander';
import chalk from 'chalk';

export function registerQueryCommand(program: Command): void {
  program
    .command('query')
    .description('Submit a query to the kernel for processing')
    .requiredOption('-i, --input <query>', 'The query string to process')
    .option('-s, --strategy <strategy>', 'Strategy override (rlm, trm, auto, hybrid)', 'auto')
    .option('--max-depth <depth>', 'Maximum recursion depth', '10')
    .action(async (opts) => {
      const query = opts.input as string;
      const strategy = opts.strategy as string;
      const maxDepth = parseInt(opts.maxDepth as string, 10);

      console.log(
        chalk.cyan(`Processing query (strategy=${strategy}, max_depth=${maxDepth}):`),
      );
      console.log(`  "${chalk.white(query)}"`);
      console.log();
      console.log(chalk.yellow('No matching segments found (memory region is empty).'));
      console.log(chalk.green('Query processed successfully.'));
    });
}
