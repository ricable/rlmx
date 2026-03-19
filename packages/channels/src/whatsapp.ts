// WhatsApp channel adapter (stub implementation).

import { generateId } from '@aix/shared';
import type { ChannelAdapter, ChannelConfig, ChannelMessage } from './types.js';

/** Stub WhatsApp adapter. */
export class WhatsAppAdapter implements ChannelAdapter {
  private apiKey: string | undefined;

  constructor(config: ChannelConfig) {
    this.apiKey = config.credentials['api_key'];
  }

  async send(msg: ChannelMessage): Promise<void> {
    if (!this.apiKey) {
      throw new Error('WhatsApp api_key not configured');
    }
  }

  async receive(): Promise<ChannelMessage> {
    if (!this.apiKey) {
      throw new Error('WhatsApp api_key not configured');
    }
    return {
      id: generateId(),
      channel: generateId(),
      sender: 'whatsapp_user',
      content: '[stub] No real messages available',
      attachments: [],
      replyTo: null,
      timestamp: new Date().toISOString(),
    };
  }

  async healthCheck(): Promise<boolean> {
    return this.apiKey !== undefined;
  }

  channelType(): string {
    return 'whatsapp';
  }
}
