//! Telegram channel adapter (stub implementation).

use crate::registry::ChannelAdapter;
use crate::types::{ChannelConfig, ChannelError, ChannelId, ChannelMessage};
use async_trait::async_trait;

/// Stub Telegram adapter that logs operations via tracing.
pub struct TelegramAdapter {
    bot_token: Option<String>,
}

impl TelegramAdapter {
    /// Create a new Telegram adapter from configuration.
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
impl ChannelAdapter for TelegramAdapter {
    async fn send(&self, msg: ChannelMessage) -> Result<(), ChannelError> {
        if self.bot_token.is_none() {
            return Err(ChannelError::AuthFailed(
                "telegram".into(),
                "bot_token not configured".into(),
            ));
        }
        tracing::info!(
            channel_type = "telegram",
            sender = %msg.sender,
            content_len = msg.content.len(),
            "stub: sending telegram message"
        );
        Ok(())
    }

    async fn receive(&self) -> Result<ChannelMessage, ChannelError> {
        if self.bot_token.is_none() {
            return Err(ChannelError::AuthFailed(
                "telegram".into(),
                "bot_token not configured".into(),
            ));
        }
        tracing::info!(channel_type = "telegram", "stub: polling telegram messages");
        Ok(ChannelMessage::text(
            ChannelId::new(),
            "telegram_user",
            "[stub] No real messages available",
        ))
    }

    async fn health_check(&self) -> bool {
        self.bot_token.is_some()
    }

    fn channel_type(&self) -> &str {
        "telegram"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn configured_config() -> ChannelConfig {
        ChannelConfig {
            channel_type: "telegram".into(),
            credentials: HashMap::from([("bot_token".into(), "test-token-123".into())]),
            enabled: true,
        }
    }

    #[tokio::test]
    async fn test_send_with_token() {
        let adapter = TelegramAdapter::new(&configured_config());
        let msg = ChannelMessage::text(ChannelId::new(), "alice", "Hello Telegram");
        assert!(adapter.send(msg).await.is_ok());
    }

    #[tokio::test]
    async fn test_send_without_token_fails() {
        let adapter = TelegramAdapter::unconfigured();
        let msg = ChannelMessage::text(ChannelId::new(), "alice", "Hello");
        assert!(adapter.send(msg).await.is_err());
    }

    #[tokio::test]
    async fn test_receive_with_token() {
        let adapter = TelegramAdapter::new(&configured_config());
        let msg = adapter.receive().await.unwrap();
        assert_eq!(msg.sender, "telegram_user");
    }

    #[tokio::test]
    async fn test_health_with_token() {
        let adapter = TelegramAdapter::new(&configured_config());
        assert!(adapter.health_check().await);
    }

    #[tokio::test]
    async fn test_health_without_token() {
        let adapter = TelegramAdapter::unconfigured();
        assert!(!adapter.health_check().await);
    }

    #[test]
    fn test_channel_type() {
        let adapter = TelegramAdapter::unconfigured();
        assert_eq!(adapter.channel_type(), "telegram");
    }
}
