// @aix/channels — Channel adapters for external messaging platforms (ADR-039).

export type { ChannelMessage, ChannelConfig, ChannelAdapter } from './types.js';

export { ChannelRegistry, ChannelNotConfiguredError } from './registry.js';

export { TelegramAdapter } from './telegram.js';
export { WhatsAppAdapter } from './whatsapp.js';
export { TeamsAdapter } from './teams.js';
export { DiscordAdapter } from './discord.js';
