// Telegram channel adapter (stub implementation).

import { generateId } from '@aix/shared';
import type { ChannelAdapter, ChannelConfig, ChannelMessage } from './types.js';

/** Stub Telegram adapter. */
export class TelegramAdapter implements ChannelAdapter {
  private botToken: string | undefined;

  constructor(config: ChannelConfig) {
    this.botToken = config.credentials['bot_token'];
  }

  async send(msg: ChannelMessage): Promise<void> {
    if (!this.botToken) {
      throw new Error('Telegram bot_token not configured');
    }
    // Stub: log would go here in real implementation
  }

  async receive(): Promise<ChannelMessage> {
    if (!this.botToken) {
      throw new Error('Telegram bot_token not configured');
    }
    return {
      id: generateId(),
      channel: generateId(),
      sender: 'telegram_user',
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
    return 'telegram';
  }
}
