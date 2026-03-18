//! IngestAdapter trait for data ingestion plugins.

use async_trait::async_trait;

use crate::context::ContextSegment;
use crate::error::PluginError;

/// Trait for adapters that ingest data from various sources into ContextSegments.
#[async_trait]
pub trait IngestAdapter: Send + Sync {
    /// Name of this ingest adapter.
    fn name(&self) -> &str;

    /// List of supported input formats (e.g., "xml", "csv", "json").
    fn supported_formats(&self) -> Vec<String>;

    /// Ingest data from a source path, returning all segments at once.
    async fn ingest(
        &self,
        source: &std::path::Path,
    ) -> Result<Vec<ContextSegment>, PluginError>;

    /// Ingest data in batches for large sources.
    async fn ingest_batch(
        &self,
        source: &std::path::Path,
        batch_size: usize,
    ) -> Result<Vec<ContextSegment>, PluginError>;
}
