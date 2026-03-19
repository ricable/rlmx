//! Plugin template - minimal "MyDomainPlugin" implementation from PRD Section 4.3.
//!
//! Use this as a starting point for creating new domain plugins.

use async_trait::async_trait;

use crate::adapter::IngestAdapter;
use crate::context::ContextSegment;
use crate::error::PluginError;
use crate::traits::DomainPlugin;
use crate::types::*;

/// A minimal template plugin for demonstration and testing.
///
/// Copy this and modify for your domain.
pub struct MyDomainPlugin {
    name: String,
    description: String,
}

impl MyDomainPlugin {
    /// Create a new template plugin with default values.
    pub fn new() -> Self {
        Self {
            name: "my-domain".to_string(),
            description: "A template domain plugin for RuVix".to_string(),
        }
    }

    /// Create a template plugin with a custom name and description.
    pub fn with_name(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

impl Default for MyDomainPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DomainPlugin for MyDomainPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> semver::Version {
        semver::Version::new(0, 1, 0)
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn ingest_adapters(&self) -> Vec<Box<dyn IngestAdapter>> {
        vec![Box::new(TemplateAdapter)]
    }

    fn strategy_preferences(&self) -> Vec<StrategyPreference> {
        vec![StrategyPreference {
            task_pattern: ".*".to_string(),
            strategy: Strategy::Auto,
            trm_model: None,
            rationale: "Default: use automatic strategy selection for all tasks.".to_string(),
        }]
    }

    fn action_extensions(&self) -> Vec<ActionDefinition> {
        vec![ActionDefinition::new("HelloWorld")
            .description("A simple hello-world action for testing.")
            .param("name", ParamType::String, true, "Name to greet")]
    }

    fn system_prompt_extension(&self) -> String {
        "You are a helpful domain assistant. Provide clear, concise answers.".to_string()
    }

    fn safety_constraints(&self) -> Vec<SafetyConstraint> {
        vec![SafetyConstraint::RateLimit {
            action: "HelloWorld".to_string(),
            max_count: 100,
            window_secs: 60,
        }]
    }
}

/// A simple template ingest adapter.
struct TemplateAdapter;

#[async_trait]
impl IngestAdapter for TemplateAdapter {
    fn name(&self) -> &str {
        "template-adapter"
    }

    fn supported_formats(&self) -> Vec<String> {
        vec!["txt".to_string(), "json".to_string()]
    }

    async fn ingest(&self, source: &std::path::Path) -> Result<Vec<ContextSegment>, PluginError> {
        let source_str = source.to_string_lossy().to_string();
        let content = tokio::fs::read_to_string(source)
            .await
            .map_err(PluginError::IoError)?;

        Ok(vec![ContextSegment::new(
            content,
            source_str,
            "my-domain".to_string(),
            "document".to_string(),
        )])
    }

    async fn ingest_batch(
        &self,
        source: &std::path::Path,
        _batch_size: usize,
    ) -> Result<Vec<ContextSegment>, PluginError> {
        self.ingest(source).await
    }
}
