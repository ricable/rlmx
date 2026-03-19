//! Core types for channel adapters.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Newtype wrapper around UUID identifying a channel instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelId(pub Uuid);

impl ChannelId {
    /// Create a new random channel ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ChannelId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ChannelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A normalized message exchanged through a channel adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    /// Unique message identifier.
    pub id: Uuid,
    /// The channel this message belongs to.
    pub channel: ChannelId,
    /// Sender identifier (platform-specific, e.g., username or user ID).
    pub sender: String,
    /// Message text content.
    pub content: String,
    /// Attachment IDs (references to external blobs).
    pub attachments: Vec<Uuid>,
    /// If this is a reply, the ID of the original message.
    pub reply_to: Option<Uuid>,
    /// When the message was created.
    pub timestamp: DateTime<Utc>,
}

impl ChannelMessage {
    /// Create a simple text message.
    pub fn text(channel: ChannelId, sender: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            channel,
            sender: sender.into(),
            content: content.into(),
            attachments: Vec::new(),
            reply_to: None,
            timestamp: Utc::now(),
        }
    }
}

/// Configuration for a channel adapter instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// The platform type (e.g., "telegram", "discord").
    pub channel_type: String,
    /// Platform-specific credentials (API tokens, webhook URLs, etc.).
    pub credentials: HashMap<String, String>,
    /// Whether this channel is enabled.
    pub enabled: bool,
}

/// Errors that can occur during channel operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ChannelError {
    /// The requested channel adapter is not configured.
    #[error("channel adapter '{0}' not configured")]
    NotConfigured(String),
    /// Authentication with the platform failed.
    #[error("authentication failed for channel '{0}': {1}")]
    AuthFailed(String, String),
    /// Sending a message failed.
    #[error("failed to send message on channel '{0}': {1}")]
    SendFailed(String, String),
    /// Receiving messages failed.
    #[error("failed to receive messages on channel '{0}': {1}")]
    ReceiveFailed(String, String),
    /// The platform rate-limited the request.
    #[error("rate limited on channel '{0}'")]
    RateLimited(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_id_new() {
        let id1 = ChannelId::new();
        let id2 = ChannelId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_channel_id_display() {
        let id = ChannelId(Uuid::nil());
        assert_eq!(id.to_string(), "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn test_channel_message_text() {
        let ch = ChannelId::new();
        let msg = ChannelMessage::text(ch, "alice", "Hello world");
        assert_eq!(msg.sender, "alice");
        assert_eq!(msg.content, "Hello world");
        assert_eq!(msg.channel, ch);
        assert!(msg.attachments.is_empty());
        assert!(msg.reply_to.is_none());
    }

    #[test]
    fn test_channel_config_serde() {
        let config = ChannelConfig {
            channel_type: "telegram".into(),
            credentials: HashMap::from([("bot_token".into(), "test-token".into())]),
            enabled: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        let back: ChannelConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.channel_type, "telegram");
        assert!(back.enabled);
    }

    #[test]
    fn test_channel_message_serde_roundtrip() {
        let msg = ChannelMessage::text(ChannelId::new(), "bob", "test message");
        let json = serde_json::to_string(&msg).unwrap();
        let back: ChannelMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(msg.id, back.id);
        assert_eq!(msg.content, back.content);
    }

    #[test]
    fn test_channel_error_display() {
        let err = ChannelError::NotConfigured("slack".into());
        assert!(err.to_string().contains("slack"));

        let err = ChannelError::RateLimited("telegram".into());
        assert!(err.to_string().contains("rate limited"));
    }

    #[test]
    fn test_channel_id_default() {
        let id = ChannelId::default();
        assert_ne!(id, ChannelId(Uuid::nil()));
    }
}
