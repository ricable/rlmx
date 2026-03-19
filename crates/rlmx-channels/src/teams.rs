//! Microsoft Teams channel adapter (stub implementation).

use crate::registry::ChannelAdapter;
use crate::types::{ChannelConfig, ChannelError, ChannelId, ChannelMessage};
use async_trait::async_trait;

/// Stub Teams adapter that logs operations via tracing.
pub struct TeamsAdapter {
    webhook_url: Option<String>,
}

impl TeamsAdapter {
    /// Create a new Teams adapter from configuration.
    pub fn new(config: &ChannelConfig) -> Self {
        Self {
            webhook_url: config.credentials.get("webhook_url").cloned(),
        }
    }

    /// Create an unconfigured adapter (for testing).
    pub fn unconfigured() -> Self {
        Self { webhook_url: None }
    }
}

#[async_trait]
impl ChannelAdapter for TeamsAdapter {
    async fn send(&self, msg: ChannelMessage) -> Result<(), ChannelError> {
        if self.webhook_url.is_none() {
            return Err(ChannelError::AuthFailed(
                "teams".into(),
                "webhook_url not configured".into(),
            ));
        }
        tracing::info!(
            channel_type = "teams",
            sender = %msg.sender,
            content_len = msg.content.len(),
            "stub: sending teams message"
        );
        Ok(())
    }

    async fn receive(&self) -> Result<ChannelMessage, ChannelError> {
        if self.webhook_url.is_none() {
            return Err(ChannelError::AuthFailed(
                "teams".into(),
                "webhook_url not configured".into(),
            ));
        }
        tracing::info!(channel_type = "teams", "stub: polling teams messages");
        Ok(ChannelMessage::text(
            ChannelId::new(),
            "teams_user",
            "[stub] No real messages available",
        ))
    }

    async fn health_check(&self) -> bool {
        self.webhook_url.is_some()
    }

    fn channel_type(&self) -> &str {
        "teams"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn configured_config() -> ChannelConfig {
        ChannelConfig {
            channel_type: "teams".into(),
            credentials: HashMap::from([(
                "webhook_url".into(),
                "https://teams.example.com/hook".into(),
            )]),
            enabled: true,
        }
    }

    #[tokio::test]
    async fn test_send_with_webhook() {
        let adapter = TeamsAdapter::new(&configured_config());
        let msg = ChannelMessage::text(ChannelId::new(), "alice", "Hello Teams");
        assert!(adapter.send(msg).await.is_ok());
    }

    #[tokio::test]
    async fn test_send_without_webhook_fails() {
        let adapter = TeamsAdapter::unconfigured();
        let msg = ChannelMessage::text(ChannelId::new(), "alice", "Hello");
        assert!(adapter.send(msg).await.is_err());
    }

    #[tokio::test]
    async fn test_health_with_webhook() {
        let adapter = TeamsAdapter::new(&configured_config());
        assert!(adapter.health_check().await);
    }

    #[tokio::test]
    async fn test_health_without_webhook() {
        let adapter = TeamsAdapter::unconfigured();
        assert!(!adapter.health_check().await);
    }

    #[test]
    fn test_channel_type() {
        let adapter = TeamsAdapter::unconfigured();
        assert_eq!(adapter.channel_type(), "teams");
    }
}
