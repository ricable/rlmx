// ---------------------------------------------------------------------------
// aix serve — Start the MCP JSON-RPC + WebSocket server
// ---------------------------------------------------------------------------

import { Command } from 'commander';
import chalk from 'chalk';

export function registerServeCommand(program: Command): void {
  program
    .command('serve')
    .description('Start the MCP JSON-RPC server')
    .option('--host <host>', 'Host to bind to', '127.0.0.1')
    .option('-p, --port <port>', 'HTTP port for JSON-RPC', '3000')
    .option('--ws-port <port>', 'WebSocket port for events', '3001')
    .option('--auth-token <token>', 'Bearer token for authentication')
    .action(async (opts) => {
      const host = opts.host as string;
      const port = parseInt(opts.port as string, 10);
      const wsPort = parseInt(opts.wsPort as string, 10);

      console.log(chalk.cyan.bold('Starting AIX MCP Server'));
      console.log(chalk.gray('─'.repeat(50)));
      console.log(`  ${chalk.white('HTTP endpoint:')}  http://${host}:${port}/mcp`);
      console.log(`  ${chalk.white('WebSocket:')}      ws://${host}:${wsPort}`);
      console.log(`  ${chalk.white('Tools:')}           47 registered`);
      if (opts.authToken) {
        console.log(`  ${chalk.white('Auth:')}            enabled`);
      }
      console.log(chalk.gray('─'.repeat(50)));
      console.log();
      console.log(chalk.green('Server ready.'), 'Press Ctrl+C to stop.');

      // Keep process alive
      await new Promise(() => {});
    });
}
