//! Plugin error types.

use thiserror::Error;

/// Errors that can occur in plugin operations.
#[derive(Error, Debug)]
pub enum PluginError {
    /// Error during data ingestion.
    #[error("Ingest error: {0}")]
    IngestError(String),

    /// Error in plugin configuration.
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Error during action execution.
    #[error("Action error: {0}")]
    ActionError(String),

    /// I/O error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    /// Plugin not found.
    #[error("Plugin not found: {0}")]
    NotFound(String),

    /// Safety constraint violation.
    #[error("Safety violation: {0}")]
    SafetyViolation(String),

    /// Unsupported format.
    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}
