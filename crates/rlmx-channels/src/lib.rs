//! Channel adapters for external messaging platforms (ADR-039).
//!
//! Provides a pluggable adapter layer that normalizes messages across
//! Telegram, WhatsApp, Microsoft Teams, and Discord. Each adapter implements
//! the [`ChannelAdapter`] trait and is registered in a [`ChannelRegistry`].

pub mod types;
pub mod registry;
pub mod telegram;
pub mod whatsapp;
pub mod teams;
pub mod discord;

pub use registry::{ChannelAdapter, ChannelRegistry};
pub use types::{ChannelConfig, ChannelError, ChannelEvent, ChannelId, ChannelMessage};
