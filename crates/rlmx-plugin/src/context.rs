//! ContextSegment shared type for plugin data exchange.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A segment of contextual data produced by an ingest adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSegment {
    /// Unique identifier for this segment.
    pub id: Uuid,
    /// Embedding vector for similarity search.
    pub embedding: Vec<f32>,
    /// The textual content of this segment.
    pub content: String,
    /// Source identifier (e.g., file path, API endpoint).
    pub source: String,
    /// Name of the plugin that produced this segment.
    pub plugin: String,
    /// Type of segment (e.g., "pm_counter", "alarm", "config").
    pub segment_type: String,
    /// Timestamp of when this segment was created or ingested.
    pub timestamp: DateTime<Utc>,
    /// Additional metadata as a JSON value.
    pub metadata: serde_json::Value,
    /// Storage tier for this segment.
    pub tier: Tier,
}

/// Storage tier classification for context segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    /// Frequently accessed, kept in fast storage.
    Hot,
    /// Occasionally accessed, standard storage.
    Warm,
    /// Rarely accessed, archived storage.
    Cold,
}

impl ContextSegment {
    /// Create a new ContextSegment with default values.
    pub fn new(content: String, source: String, plugin: String, segment_type: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            embedding: Vec::new(),
            content,
            source,
            plugin,
            segment_type,
            timestamp: Utc::now(),
            metadata: serde_json::Value::Null,
            tier: Tier::Hot,
        }
    }

    /// Set the embedding vector.
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = embedding;
        self
    }

    /// Set the metadata.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set the tier.
    pub fn with_tier(mut self, tier: Tier) -> Self {
        self.tier = tier;
        self
    }
}

impl Default for Tier {
    fn default() -> Self {
        Tier::Hot
    }
}
