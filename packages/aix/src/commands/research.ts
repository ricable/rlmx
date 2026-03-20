// ---------------------------------------------------------------------------
// aix research — Manage auto-research experiments
// ---------------------------------------------------------------------------

import { Command } from 'commander';
import chalk from 'chalk';
import { randomUUID } from 'node:crypto';

export function registerResearchCommand(program: Command): void {
  const research = program
    .command('research')
    .description('Manage auto-research experiments');

  research
    .command('start')
    .description('Start a research experiment')
    .requiredOption('--topic <topic>', 'Research topic')
    .option('--hypotheses <count>', 'Number of hypotheses to test', '3')
    .option('--nodes <count>', 'Number of nodes to use', '1')
    .action(async (opts) => {
      const topic = opts.topic as string;
      const hypotheses = parseInt(opts.hypotheses as string, 10);
      const nodes = parseInt(opts.nodes as string, 10);
      const researchId = randomUUID();

      console.log(chalk.cyan.bold('AIX Auto-Research Pipeline'));
      console.log(chalk.gray('─'.repeat(50)));
      console.log(`  ${chalk.white('Research ID:')}  ${researchId}`);
      console.log(`  ${chalk.white('Topic:')}        ${topic}`);
      console.log(`  ${chalk.white('Hypotheses:')}   ${hypotheses}`);
      console.log(`  ${chalk.white('Nodes:')}        ${nodes}`);
      console.log(chalk.gray('─'.repeat(50)));
      console.log();
      console.log(chalk.green('Research started.'));
    });

  research
    .command('status <researchId>')
    .description('Check research status')
    .action(async (researchId: string) => {
      console.log(chalk.cyan('Research Status:'));
      console.log(`  ${chalk.white('id:')}     ${researchId}`);
      console.log(`  ${chalk.white('status:')} running`);
      console.log(`  ${chalk.white('tested:')} 0/3 hypotheses`);
    });

  research
    .command('list')
    .description('List all research tasks')
    .action(async () => {
      console.log(chalk.cyan('Research Tasks:'));
      console.log(chalk.yellow('  No active research tasks.'));
    });
}
