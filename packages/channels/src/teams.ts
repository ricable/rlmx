// Microsoft Teams channel adapter (stub implementation).

import { generateId } from '@aix/shared';
import type { ChannelAdapter, ChannelConfig, ChannelMessage } from './types.js';

/** Stub Teams adapter. */
export class TeamsAdapter implements ChannelAdapter {
  private webhookUrl: string | undefined;

  constructor(config: ChannelConfig) {
    this.webhookUrl = config.credentials['webhook_url'];
  }

  async send(msg: ChannelMessage): Promise<void> {
    if (!this.webhookUrl) {
      throw new Error('Teams webhook_url not configured');
    }
  }

  async receive(): Promise<ChannelMessage> {
    if (!this.webhookUrl) {
      throw new Error('Teams webhook_url not configured');
    }
    return {
      id: generateId(),
      channel: generateId(),
      sender: 'teams_user',
      content: '[stub] No real messages available',
      attachments: [],
      replyTo: null,
      timestamp: new Date().toISOString(),
    };
  }

  async healthCheck(): Promise<boolean> {
    return this.webhookUrl !== undefined;
  }

  channelType(): string {
    return 'teams';
  }
}
