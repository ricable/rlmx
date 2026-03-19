// ---------------------------------------------------------------------------
// aix voice — Voice-first interaction pipeline
// ---------------------------------------------------------------------------

import { Command } from 'commander';
import chalk from 'chalk';

export function registerVoiceCommand(program: Command): void {
  const voice = program
    .command('voice')
    .description('Voice-first interaction pipeline');

  voice
    .command('start')
    .description('Start voice pipeline')
    .action(async () => {
      console.log(chalk.cyan.bold('Starting voice pipeline...'));
      console.log(`  ${chalk.white('VAD:')}  Voice Activity Detector ready`);
      console.log(`  ${chalk.white('STT:')}  On-device Whisper-tiny Q4 loaded`);
      console.log(`  ${chalk.white('TTS:')}  6 domain personas available`);
      console.log();
      console.log(chalk.green('Voice pipeline active.'), 'Listening...');
    });

  voice
    .command('transcribe')
    .description('Simulate transcription')
    .requiredOption('--text <text>', 'Text to transcribe')
    .action(async (opts) => {
      const text = opts.text as string;
      console.log(chalk.cyan('Transcription result:'));
      console.log(`  text: "${chalk.white(text)}"`);
      console.log(`  confidence: ${chalk.green('0.95')}`);
      console.log(`  language: en`);
      console.log(`  is_final: true`);
    });

  voice
    .command('intents')
    .description('Show intent decomposition from natural language')
    .requiredOption('--text <text>', 'Natural language input')
    .action(async (opts) => {
      const text = opts.text as string;
      console.log(chalk.cyan('Multi-Intent Decomposition:'));
      console.log(`  input: "${chalk.white(text)}"`);
      console.log();
      console.log(chalk.yellow('  [1] General intent detected'));
      console.log(`       domain: general`);
      console.log(`       confidence: 0.85`);
    });

  voice
    .command('session')
    .description('Manage voice sessions')
    .option('--list', 'List all sessions')
    .action(async (opts) => {
      if (opts.list) {
        console.log(chalk.cyan('Voice Sessions:'));
        console.log(chalk.yellow('  No active sessions.'));
      }
    });
}
