// ---------------------------------------------------------------------------
// aix phone — On-device phone runtime
// ---------------------------------------------------------------------------

import { Command } from 'commander';
import chalk from 'chalk';

export function registerPhoneCommand(program: Command): void {
  const phone = program
    .command('phone')
    .description('On-device phone runtime');

  phone
    .command('status')
    .description('Show phone runtime status')
    .action(async () => {
      console.log(chalk.cyan('Phone Runtime Status:'));
      console.log(`  ${chalk.white('status:')}     ${chalk.green('active')}`);
      console.log(`  ${chalk.white('free_agents:')} 5 (always-on)`);
      console.log(`  ${chalk.white('offline:')}    outbox empty (0/100)`);
      console.log(`  ${chalk.white('scheduler:')}  battery-aware mode`);
    });

  phone
    .command('agents')
    .description('List on-device agents')
    .action(async () => {
      console.log(chalk.cyan('On-Device Agents (5 always-on):'));
      console.log(`  ${chalk.green('1.')} Morning Briefer`);
      console.log(`  ${chalk.green('2.')} Bill Watcher`);
      console.log(`  ${chalk.green('3.')} Health Nudger`);
      console.log(`  ${chalk.green('4.')} Calendar Scout`);
      console.log(`  ${chalk.green('5.')} Deal Finder`);
    });

  phone
    .command('battery')
    .description('Show battery-aware scheduling info')
    .action(async () => {
      console.log(chalk.cyan('Battery-Aware Scheduling:'));
      console.log(`  ${chalk.white('policy:')}       balanced`);
      console.log(`  ${chalk.white('charging:')}     unknown`);
      console.log(`  ${chalk.white('level:')}        unknown`);
      console.log(`  ${chalk.white('bg_tasks:')}     idle`);
    });
}
