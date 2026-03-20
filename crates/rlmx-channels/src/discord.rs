//! Discord channel adapter (stub implementation).

use crate::registry::ChannelAdapter;
use crate::types::{ChannelConfig, ChannelError, ChannelId, ChannelMessage};
use async_trait::async_trait;

/// Stub Discord adapter that logs operations via tracing.
pub struct DiscordAdapter {
    bot_token: Option<String>,
}

impl DiscordAdapter {
    /// Create a new Discord adapter from configuration.
    pub fn new(config: &ChannelConfig) -> Self {
        Self {
            bot_token: config.credentials.get("bot_token").cloned(),
        }
    }

    /// Create an unconfigured adapter (for testing).
    pub fn unconfigured() -> Self {
        Self { bot_token: None }
    }
}

#[async_trait]
impl ChannelAdapter for DiscordAdapter {
    async fn send(&self, msg: ChannelMessage) -> Result<(), ChannelError> {
        if self.bot_token.is_none() {
            return Err(ChannelError::AuthFailed(
                "discord".into(),
                "bot_token not configured".into(),
            ));
        }
        tracing::info!(
            channel_type = "discord",
            sender = %msg.sender,
            content_len = msg.content.len(),
            "stub: sending discord message"
        );
        Ok(())
    }

    async fn receive(&self) -> Result<ChannelMessage, ChannelError> {
        if self.bot_token.is_none() {
            return Err(ChannelError::AuthFailed(
                "discord".into(),
                "bot_token not configured".into(),
            ));
        }
        tracing::info!(channel_type = "discord", "stub: polling discord messages");
        Ok(ChannelMessage::text(
            ChannelId::new(),
            "discord_user",
            "[stub] No real messages available",
        ))
    }

    async fn health_check(&self) -> bool {
        self.bot_token.is_some()
    }

    fn channel_type(&self) -> &str {
        "discord"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn configured_config() -> ChannelConfig {
        ChannelConfig {
            channel_type: "discord".into(),
            credentials: HashMap::from([("bot_token".into(), "discord-token-123".into())]),
            enabled: true,
        }
    }

    #[tokio::test]
    async fn test_send_with_token() {
        let adapter = DiscordAdapter::new(&configured_config());
        let msg = ChannelMessage::text(ChannelId::new(), "alice", "Hello Discord");
        assert!(adapter.send(msg).await.is_ok());
    }

    #[tokio::test]
    async fn test_send_without_token_fails() {
        let adapter = DiscordAdapter::unconfigured();
        let msg = ChannelMessage::text(ChannelId::new(), "alice", "Hello");
        assert!(adapter.send(msg).await.is_err());
    }

    #[tokio::test]
    async fn test_receive_with_token() {
        let adapter = DiscordAdapter::new(&configured_config());
        let msg = adapter.receive().await.unwrap();
        assert_eq!(msg.sender, "discord_user");
    }

    #[tokio::test]
    async fn test_health_with_token() {
        let adapter = DiscordAdapter::new(&configured_config());
        assert!(adapter.health_check().await);
    }

    #[tokio::test]
    async fn test_health_without_token() {
        let adapter = DiscordAdapter::unconfigured();
        assert!(!adapter.health_check().await);
    }

    #[test]
    fn test_channel_type() {
        let adapter = DiscordAdapter::unconfigured();
        assert_eq!(adapter.channel_type(), "discord");
    }
}
