import { describe, it, expect, beforeEach } from 'vitest';
import {
  ChannelRegistry,
  ChannelNotConfiguredError,
  TelegramAdapter,
  WhatsAppAdapter,
  TeamsAdapter,
  DiscordAdapter,
} from '../src/index.js';
import type { ChannelAdapter, ChannelConfig, ChannelMessage } from '../src/index.js';
import { generateId } from '@aix/shared';

function makeMessage(sender = 'test-user', content = 'hello'): ChannelMessage {
  return {
    id: generateId(),
    channel: generateId(),
    sender,
    content,
    attachments: [],
    replyTo: null,
    timestamp: new Date().toISOString(),
  };
}

describe('ChannelRegistry', () => {
  let registry: ChannelRegistry;

  beforeEach(() => {
    registry = new ChannelRegistry();
  });

  it('should register and list adapters', () => {
    const config: ChannelConfig = {
      channelType: 'telegram',
      credentials: { bot_token: 'test-token' },
      enabled: true,
    };
    registry.register('telegram', new TelegramAdapter(config));
    expect(registry.list()).toContain('telegram');
  });

  it('should send through registered adapter', async () => {
    const config: ChannelConfig = {
      channelType: 'telegram',
      credentials: { bot_token: 'test-token' },
      enabled: true,
    };
    registry.register('telegram', new TelegramAdapter(config));
    await expect(registry.send('telegram', makeMessage())).resolves.toBeUndefined();
  });

  it('should throw when sending to unregistered adapter', async () => {
    await expect(registry.send('nonexistent', makeMessage())).rejects.toThrow(
      ChannelNotConfiguredError,
    );
  });

  it('should check health of registered adapter', async () => {
    const config: ChannelConfig = {
      channelType: 'telegram',
      credentials: { bot_token: 'test-token' },
      enabled: true,
    };
    registry.register('telegram', new TelegramAdapter(config));
    await expect(registry.health('telegram')).resolves.toBe(true);
  });

  it('should throw when checking health of unregistered adapter', async () => {
    await expect(registry.health('nonexistent')).rejects.toThrow(ChannelNotConfiguredError);
  });

  it('should remove adapter', () => {
    const config: ChannelConfig = {
      channelType: 'telegram',
      credentials: { bot_token: 'test-token' },
      enabled: true,
    };
    registry.register('telegram', new TelegramAdapter(config));
    expect(registry.remove('telegram')).toBe(true);
    expect(registry.contains('telegram')).toBe(false);
  });

  it('should return false when removing nonexistent', () => {
    expect(registry.remove('nonexistent')).toBe(false);
  });

  it('should check contains', () => {
    expect(registry.contains('telegram')).toBe(false);
    const config: ChannelConfig = {
      channelType: 'telegram',
      credentials: { bot_token: 'test-token' },
      enabled: true,
    };
    registry.register('telegram', new TelegramAdapter(config));
    expect(registry.contains('telegram')).toBe(true);
  });
});

describe('TelegramAdapter', () => {
  it('should send with token', async () => {
    const adapter = new TelegramAdapter({
      channelType: 'telegram',
      credentials: { bot_token: 'test' },
      enabled: true,
    });
    await expect(adapter.send(makeMessage())).resolves.toBeUndefined();
  });

  it('should fail send without token', async () => {
    const adapter = new TelegramAdapter({
      channelType: 'telegram',
      credentials: {},
      enabled: true,
    });
    await expect(adapter.send(makeMessage())).rejects.toThrow('bot_token');
  });

  it('should report healthy with token', async () => {
    const adapter = new TelegramAdapter({
      channelType: 'telegram',
      credentials: { bot_token: 'test' },
      enabled: true,
    });
    expect(await adapter.healthCheck()).toBe(true);
  });

  it('should report unhealthy without token', async () => {
    const adapter = new TelegramAdapter({
      channelType: 'telegram',
      credentials: {},
      enabled: true,
    });
    expect(await adapter.healthCheck()).toBe(false);
  });

  it('should return channel type', () => {
    const adapter = new TelegramAdapter({
      channelType: 'telegram',
      credentials: {},
      enabled: true,
    });
    expect(adapter.channelType()).toBe('telegram');
  });
});

describe('WhatsAppAdapter', () => {
  it('should send with api_key', async () => {
    const adapter = new WhatsAppAdapter({
      channelType: 'whatsapp',
      credentials: { api_key: 'test' },
      enabled: true,
    });
    await expect(adapter.send(makeMessage())).resolves.toBeUndefined();
  });

  it('should fail without api_key', async () => {
    const adapter = new WhatsAppAdapter({
      channelType: 'whatsapp',
      credentials: {},
      enabled: true,
    });
    await expect(adapter.send(makeMessage())).rejects.toThrow('api_key');
  });

  it('should return channel type', () => {
    const adapter = new WhatsAppAdapter({
      channelType: 'whatsapp',
      credentials: {},
      enabled: true,
    });
    expect(adapter.channelType()).toBe('whatsapp');
  });
});

describe('TeamsAdapter', () => {
  it('should send with webhook_url', async () => {
    const adapter = new TeamsAdapter({
      channelType: 'teams',
      credentials: { webhook_url: 'https://example.com' },
      enabled: true,
    });
    await expect(adapter.send(makeMessage())).resolves.toBeUndefined();
  });

  it('should fail without webhook_url', async () => {
    const adapter = new TeamsAdapter({
      channelType: 'teams',
      credentials: {},
      enabled: true,
    });
    await expect(adapter.send(makeMessage())).rejects.toThrow('webhook_url');
  });

  it('should return channel type', () => {
    const adapter = new TeamsAdapter({
      channelType: 'teams',
      credentials: {},
      enabled: true,
    });
    expect(adapter.channelType()).toBe('teams');
  });
});

describe('DiscordAdapter', () => {
  it('should send with bot_token', async () => {
    const adapter = new DiscordAdapter({
      channelType: 'discord',
      credentials: { bot_token: 'test' },
      enabled: true,
    });
    await expect(adapter.send(makeMessage())).resolves.toBeUndefined();
  });

  it('should fail without bot_token', async () => {
    const adapter = new DiscordAdapter({
      channelType: 'discord',
      credentials: {},
      enabled: true,
    });
    await expect(adapter.send(makeMessage())).rejects.toThrow('bot_token');
  });

  it('should receive stub message', async () => {
    const adapter = new DiscordAdapter({
      channelType: 'discord',
      credentials: { bot_token: 'test' },
      enabled: true,
    });
    const msg = await adapter.receive();
    expect(msg.sender).toBe('discord_user');
  });

  it('should return channel type', () => {
    const adapter = new DiscordAdapter({
      channelType: 'discord',
      credentials: {},
      enabled: true,
    });
    expect(adapter.channelType()).toBe('discord');
  });
});
