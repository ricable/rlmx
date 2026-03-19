// ---------------------------------------------------------------------------
// aix engagement — Engagement and gamification
// ---------------------------------------------------------------------------

import { Command } from 'commander';
import chalk from 'chalk';

export function registerEngagementCommand(program: Command): void {
  const engagement = program
    .command('engagement')
    .description('Engagement and gamification');

  engagement
    .command('score')
    .description('Show overall Life Score')
    .action(async () => {
      console.log(chalk.cyan('Life Score:'));
      console.log(`  ${chalk.white('score:')}  ${chalk.green('30')} / 100 (floor)`);
      console.log(`  ${chalk.white('trend:')}  neutral`);
      console.log(chalk.gray('  Composite of 12 life domain scores.'));
    });

  engagement
    .command('savings')
    .description('Show money saved counter')
    .action(async () => {
      console.log(chalk.cyan('Money Saved:'));
      console.log(`  ${chalk.white('total:')}  ${chalk.green('$0.00')}`);
      console.log(chalk.gray('  Savings require ProofSeal verification.'));
    });

  engagement
    .command('streak')
    .description('Show current streak')
    .action(async () => {
      console.log(chalk.cyan('Streak:'));
      console.log(`  ${chalk.white('current:')}  ${chalk.yellow('0 days')}`);
      console.log(`  ${chalk.white('longest:')}  0 days`);
      console.log(`  ${chalk.white('freeze:')}   available (max 1 per 30 days)`);
    });

  engagement
    .command('achievements')
    .description('List achievements')
    .action(async () => {
      console.log(chalk.cyan('Achievements:'));
      console.log(chalk.yellow('  0 / 50 unlocked'));
      console.log(chalk.gray('  Start using AIX to unlock achievements!'));
    });
}
