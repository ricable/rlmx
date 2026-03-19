// Discord channel adapter (stub implementation).

import { generateId } from '@aix/shared';
import type { ChannelAdapter, ChannelConfig, ChannelMessage } from './types.js';

/** Stub Discord adapter. */
export class DiscordAdapter implements ChannelAdapter {
  private botToken: string | undefined;

  constructor(config: ChannelConfig) {
    this.botToken = config.credentials['bot_token'];
  }

  async send(msg: ChannelMessage): Promise<void> {
    if (!this.botToken) {
      throw new Error('Discord bot_token not configured');
    }
  }

  async receive(): Promise<ChannelMessage> {
    if (!this.botToken) {
      throw new Error('Discord bot_token not configured');
    }
    return {
      id: generateId(),
      channel: generateId(),
      sender: 'discord_user',
      content: '[stub] No real messages available',
      attachments: [],
      replyTo: null,
      timestamp: new Date().toISOString(),
    };
  }

  async healthCheck(): Promise<boolean> {
    return this.botToken !== undefined;
  }

  channelType(): string {
    return 'discord';
  }
}
