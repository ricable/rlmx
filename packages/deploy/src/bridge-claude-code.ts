/**
 * Claude Code CLI bridge adapter.
 *
 * Spawns the `claude` CLI subprocess to send prompts and capture responses.
 */

import { exec } from 'node:child_process';
import { promisify } from 'node:util';
import type { TransportAdapter, AgentRequest, AgentResponse } from './transport.js';
import type { BridgeConfig } from './manifest.js';

const execAsync = promisify(exec);

/**
 * Bridge adapter that delegates inference to the Claude Code CLI (`claude`).
 *
 * - `send()` spawns `claude -p <prompt>` and captures stdout.
 * - `healthCheck()` verifies the `claude` command is available via `--version`.
 */
export class ClaudeCodeBridgeAdapter implements TransportAdapter {
  private command: string;
  private model?: string;

  constructor(config: BridgeConfig) {
    this.command = config.command ?? 'claude';
    this.model = config.model;
  }

  async send(request: AgentRequest): Promise<AgentResponse> {
    try {
      const prompt = typeof request.params === 'string'
        ? request.params
        : JSON.stringify(request.params ?? request.method);

      const args: string[] = ['-p', JSON.stringify(prompt)];
      if (this.model) {
        args.push('--model', this.model);
      }

      const cmd = `${this.command} ${args.join(' ')}`;

      const timeoutMs = request.timeout ?? 120_000;
      const { stdout, stderr } = await execAsync(cmd, {
        timeout: timeoutMs,
        maxBuffer: 10 * 1024 * 1024,
        env: { ...process.env },
      });

      if (stderr && !stdout) {
        return { status: 'error', error: stderr.trim() };
      }

      return { status: 'ok', data: stdout.trim() };
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      if (message.includes('ENOENT') || message.includes('not found')) {
        return { status: 'unavailable', error: `claude CLI not found: ${message}` };
      }
      return { status: 'error', error: message };
    }
  }

  async healthCheck(): Promise<boolean> {
    try {
      await execAsync(`${this.command} --version`, { timeout: 10_000 });
      return true;
    } catch {
      return false;
    }
  }

  protocol(): string {
    return 'bridge:claude-code';
  }
}
