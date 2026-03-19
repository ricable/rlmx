//! Channel adapter registry — routes messages to platform adapters.

use crate::types::{ChannelError, ChannelMessage};
use async_trait::async_trait;
use std::collections::HashMap;

/// Trait that all channel adapters must implement.
#[async_trait]
pub trait ChannelAdapter: Send + Sync {
    /// Send a message through this channel.
    async fn send(&self, msg: ChannelMessage) -> Result<(), ChannelError>;
    /// Receive the next available message from this channel.
    async fn receive(&self) -> Result<ChannelMessage, ChannelError>;
    /// Check if the adapter is healthy and properly configured.
    async fn health_check(&self) -> bool;
    /// Return the platform type identifier.
    fn channel_type(&self) -> &str;
}

/// Registry of named channel adapters.
pub struct ChannelRegistry {
    adapters: HashMap<String, Box<dyn ChannelAdapter>>,
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    /// Register a named adapter.
    pub fn register(&mut self, name: impl Into<String>, adapter: Box<dyn ChannelAdapter>) {
        self.adapters.insert(name.into(), adapter);
    }

    /// Send a message through a named adapter.
    pub async fn send(&self, name: &str, msg: ChannelMessage) -> Result<(), ChannelError> {
        let adapter = self
            .adapters
            .get(name)
            .ok_or_else(|| ChannelError::NotConfigured(name.to_string()))?;
        adapter.send(msg).await
    }

    /// List all registered adapter names.
    pub fn list(&self) -> Vec<&str> {
        self.adapters.keys().map(|s| s.as_str()).collect()
    }

    /// Check health of a named adapter.
    pub async fn health(&self, name: &str) -> Result<bool, ChannelError> {
        let adapter = self
            .adapters
            .get(name)
            .ok_or_else(|| ChannelError::NotConfigured(name.to_string()))?;
        Ok(adapter.health_check().await)
    }

    /// Remove an adapter by name.
    pub fn remove(&mut self, name: &str) -> bool {
        self.adapters.remove(name).is_some()
    }

    /// Check if an adapter is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.adapters.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ChannelId;

    struct MockAdapter {
        healthy: bool,
    }

    #[async_trait]
    impl ChannelAdapter for MockAdapter {
        async fn send(&self, _msg: ChannelMessage) -> Result<(), ChannelError> {
            Ok(())
        }
        async fn receive(&self) -> Result<ChannelMessage, ChannelError> {
            Ok(ChannelMessage::text(
                ChannelId::new(),
                "mock",
                "mock message",
            ))
        }
        async fn health_check(&self) -> bool {
            self.healthy
        }
        fn channel_type(&self) -> &str {
            "mock"
        }
    }

    #[tokio::test]
    async fn test_register_and_send() {
        let mut reg = ChannelRegistry::new();
        reg.register("mock", Box::new(MockAdapter { healthy: true }));
        let msg = ChannelMessage::text(ChannelId::new(), "test", "hello");
        assert!(reg.send("mock", msg).await.is_ok());
    }

    #[tokio::test]
    async fn test_send_unregistered_returns_error() {
        let reg = ChannelRegistry::new();
        let msg = ChannelMessage::text(ChannelId::new(), "test", "hello");
        assert!(reg.send("nonexistent", msg).await.is_err());
    }

    #[test]
    fn test_list_adapters() {
        let mut reg = ChannelRegistry::new();
        reg.register("telegram", Box::new(MockAdapter { healthy: true }));
        reg.register("discord", Box::new(MockAdapter { healthy: true }));
        let list = reg.list();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_health_check() {
        let mut reg = ChannelRegistry::new();
        reg.register("healthy", Box::new(MockAdapter { healthy: true }));
        reg.register("unhealthy", Box::new(MockAdapter { healthy: false }));
        assert!(reg.health("healthy").await.unwrap());
        assert!(!reg.health("unhealthy").await.unwrap());
    }

    #[tokio::test]
    async fn test_health_unregistered_returns_error() {
        let reg = ChannelRegistry::new();
        assert!(reg.health("nonexistent").await.is_err());
    }

    #[test]
    fn test_remove_adapter() {
        let mut reg = ChannelRegistry::new();
        reg.register("mock", Box::new(MockAdapter { healthy: true }));
        assert!(reg.remove("mock"));
        assert!(!reg.contains("mock"));
    }

    #[test]
    fn test_remove_nonexistent() {
        let mut reg = ChannelRegistry::new();
        assert!(!reg.remove("nonexistent"));
    }

    #[test]
    fn test_contains() {
        let mut reg = ChannelRegistry::new();
        reg.register("mock", Box::new(MockAdapter { healthy: true }));
        assert!(reg.contains("mock"));
        assert!(!reg.contains("other"));
    }

    #[test]
    fn test_default() {
        let reg = ChannelRegistry::default();
        assert!(reg.list().is_empty());
    }
}
