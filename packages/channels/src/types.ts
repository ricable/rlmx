// Channel adapter types (ADR-039).
// Mirrors rlmx-channels Rust types.

/** A normalized message exchanged through a channel adapter. */
export interface ChannelMessage {
  /** Unique message identifier. */
  id: string;
  /** Channel identifier. */
  channel: string;
  /** Sender identifier (platform-specific). */
  sender: string;
  /** Message text content. */
  content: string;
  /** Attachment IDs. */
  attachments: string[];
  /** Reply-to message ID (null if not a reply). */
  replyTo: string | null;
  /** ISO 8601 timestamp. */
  timestamp: string;
}

/** Configuration for a channel adapter instance. */
export interface ChannelConfig {
  /** Platform type (e.g., "telegram", "discord"). */
  channelType: string;
  /** Platform-specific credentials. */
  credentials: Record<string, string>;
  /** Whether this channel is enabled. */
  enabled: boolean;
}

/** Interface that all channel adapters must implement. */
export interface ChannelAdapter {
  /** Send a message through this channel. */
  send(msg: ChannelMessage): Promise<void>;
  /** Receive the next available message. */
  receive(): Promise<ChannelMessage>;
  /** Check if the adapter is healthy. */
  healthCheck(): Promise<boolean>;
  /** Return the platform type identifier. */
  channelType(): string;
}
