//! WhatsApp channel adapter (stub implementation).

use crate::registry::ChannelAdapter;
use crate::types::{ChannelConfig, ChannelError, ChannelId, ChannelMessage};
use async_trait::async_trait;

/// Stub WhatsApp adapter that logs operations via tracing.
pub struct WhatsAppAdapter {
    api_key: Option<String>,
}

impl WhatsAppAdapter {
    /// Create a new WhatsApp adapter from configuration.
    pub fn new(config: &ChannelConfig) -> Self {
        Self {
            api_key: config.credentials.get("api_key").cloned(),
        }
    }

    /// Create an unconfigured adapter (for testing).
    pub fn unconfigured() -> Self {
        Self { api_key: None }
    }
}

#[async_trait]
impl ChannelAdapter for WhatsAppAdapter {
    async fn send(&self, msg: ChannelMessage) -> Result<(), ChannelError> {
        if self.api_key.is_none() {
            return Err(ChannelError::AuthFailed(
                "whatsapp".into(),
                "api_key not configured".into(),
            ));
        }
        tracing::info!(
            channel_type = "whatsapp",
            sender = %msg.sender,
            content_len = msg.content.len(),
            "stub: sending whatsapp message"
        );
        Ok(())
    }

    async fn receive(&self) -> Result<ChannelMessage, ChannelError> {
        if self.api_key.is_none() {
            return Err(ChannelError::AuthFailed(
                "whatsapp".into(),
                "api_key not configured".into(),
            ));
        }
        tracing::info!(channel_type = "whatsapp", "stub: polling whatsapp messages");
        Ok(ChannelMessage::text(
            ChannelId::new(),
            "whatsapp_user",
            "[stub] No real messages available",
        ))
    }

    async fn health_check(&self) -> bool {
        self.api_key.is_some()
    }

    fn channel_type(&self) -> &str {
        "whatsapp"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn configured_config() -> ChannelConfig {
        ChannelConfig {
            channel_type: "whatsapp".into(),
            credentials: HashMap::from([("api_key".into(), "wa-key-123".into())]),
            enabled: true,
        }
    }

    #[tokio::test]
    async fn test_send_with_key() {
        let adapter = WhatsAppAdapter::new(&configured_config());
        let msg = ChannelMessage::text(ChannelId::new(), "alice", "Hello WhatsApp");
        assert!(adapter.send(msg).await.is_ok());
    }

    #[tokio::test]
    async fn test_send_without_key_fails() {
        let adapter = WhatsAppAdapter::unconfigured();
        let msg = ChannelMessage::text(ChannelId::new(), "alice", "Hello");
        assert!(adapter.send(msg).await.is_err());
    }

    #[tokio::test]
    async fn test_health_with_key() {
        let adapter = WhatsAppAdapter::new(&configured_config());
        assert!(adapter.health_check().await);
    }

    #[tokio::test]
    async fn test_health_without_key() {
        let adapter = WhatsAppAdapter::unconfigured();
        assert!(!adapter.health_check().await);
    }

    #[test]
    fn test_channel_type() {
        let adapter = WhatsAppAdapter::unconfigured();
        assert_eq!(adapter.channel_type(), "whatsapp");
    }
}
