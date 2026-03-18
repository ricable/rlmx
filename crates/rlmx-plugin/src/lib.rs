//! rlmx-plugin: Plugin architecture for the RuVix kernel.
//!
//! This crate defines the plugin system including:
//! - The `DomainPlugin` trait for extending the kernel with domain capabilities
//! - Type definitions for strategies, actions, safety constraints, and more
//! - The `IngestAdapter` trait for data ingestion
//! - The `DomainEvaluator` trait for response evaluation
//! - A plugin registry for loading and managing plugins
//! - A safety engine for constraint enforcement
//! - The Ericsson RAN reference plugin implementation
//! - A template plugin for quick starts

pub mod adapter;
pub mod context;
pub mod ericsson;
pub mod error;
pub mod evaluator;
pub mod loader;
pub mod safety;
pub mod template;
pub mod traits;
pub mod types;

// Re-exports for convenient access.
pub use adapter::IngestAdapter;
pub use context::{ContextSegment, Tier};
pub use ericsson::EricssonRanPlugin;
pub use error::PluginError;
pub use evaluator::{DomainEvaluator, EvaluationResult};
pub use loader::{PluginInfo, PluginRegistry};
pub use safety::{SafetyEngine, SafetyResult};
pub use template::MyDomainPlugin;
pub use traits::DomainPlugin;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Plugin Registration Tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_plugin_registration() {
        let mut registry = PluginRegistry::new();
        let plugin = Box::new(EricssonRanPlugin::new());
        assert!(registry.register(plugin).is_ok());
        assert_eq!(registry.len(), 1);
        assert!(registry.get("ericsson-ran").is_some());
    }

    #[test]
    fn test_plugin_retrieval() {
        let mut registry = PluginRegistry::new();
        registry
            .register(Box::new(EricssonRanPlugin::new()))
            .unwrap();
        registry.register(Box::new(MyDomainPlugin::new())).unwrap();

        let ericsson = registry.get("ericsson-ran").unwrap();
        assert_eq!(ericsson.name(), "ericsson-ran");

        let template = registry.get("my-domain").unwrap();
        assert_eq!(template.name(), "my-domain");

        assert!(registry.get("nonexistent").is_none());

        let list = registry.list();
        assert_eq!(list.len(), 2);
    }

    // -----------------------------------------------------------------------
    // Ericsson Plugin Metadata Tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_ericsson_plugin_metadata() {
        let plugin = EricssonRanPlugin::new();
        assert_eq!(plugin.name(), "ericsson-ran");
        assert_eq!(plugin.version(), semver::Version::new(0, 1, 0));
        assert!(plugin.description().contains("Ericsson"));
        assert!(plugin.description().contains("Radio Access Network"));
    }

    #[test]
    fn test_ericsson_plugin_capabilities() {
        let plugin = EricssonRanPlugin::new();

        // 6 ingest adapters.
        let adapters = plugin.ingest_adapters();
        assert_eq!(adapters.len(), 6);
        let adapter_names: Vec<&str> = adapters.iter().map(|a| a.name()).collect();
        assert!(adapter_names.contains(&"pm-xml"));
        assert!(adapter_names.contains(&"csv-counter"));
        assert!(adapter_names.contains(&"fm-alarm"));
        assert!(adapter_names.contains(&"cm-export"));
        assert!(adapter_names.contains(&"e2sm-kpm"));
        assert!(adapter_names.contains(&"pm-api"));

        // 4 strategy preferences.
        let strategies = plugin.strategy_preferences();
        assert_eq!(strategies.len(), 4);

        // 3 action extensions.
        let actions = plugin.action_extensions();
        assert_eq!(actions.len(), 3);
        let action_names: Vec<&str> = actions.iter().map(|a| a.name.as_str()).collect();
        assert!(action_names.contains(&"OptimizeParameter"));
        assert!(action_names.contains(&"FeatureLookup"));
        assert!(action_names.contains(&"ParameterValidate"));

        // 2 TRM models.
        let models = plugin.trm_models();
        assert_eq!(models.len(), 2);

        // 6 safety constraints.
        let constraints = plugin.safety_constraints();
        assert_eq!(constraints.len(), 6);

        // Graph schema with 3 node types and 3 edge types.
        let schema = plugin.graph_schema().unwrap();
        assert_eq!(schema.node_types.len(), 3);
        assert_eq!(schema.edge_types.len(), 3);

        // Embedding config.
        let embed = plugin.embedding_config().unwrap();
        assert_eq!(embed.dimensions, 768);

        // Distance metric.
        assert_eq!(plugin.distance_metric(), Some(DistanceMetric::Cosine));

        // System prompt extension.
        let prompt = plugin.system_prompt_extension();
        assert!(prompt.contains("Ericsson"));
        assert!(prompt.contains("3GPP"));

        // Evaluator is present.
        assert!(plugin.evaluator().is_some());
    }

    // -----------------------------------------------------------------------
    // Safety Engine Tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_safety_engine_allows_valid_params() {
        let engine = SafetyEngine::new();
        let constraints = vec![SafetyConstraint::ParameterBound {
            param: "cio".to_string(),
            min: -24.0,
            max: 24.0,
        }];

        let params = serde_json::json!({ "cio": 5.0 });
        let result = engine.check("OptimizeParameter", &params, &constraints);
        assert_eq!(result, SafetyResult::Allowed);
    }

    #[test]
    fn test_safety_engine_rejects_out_of_bounds() {
        let engine = SafetyEngine::new();
        let constraints = vec![SafetyConstraint::ParameterBound {
            param: "cio".to_string(),
            min: -24.0,
            max: 24.0,
        }];

        let params = serde_json::json!({ "cio": 30.0 });
        let result = engine.check("OptimizeParameter", &params, &constraints);
        match result {
            SafetyResult::Rejected { reason } => {
                assert!(reason.contains("cio"));
                assert!(reason.contains("30"));
            }
            other => panic!("Expected Rejected, got {:?}", other),
        }
    }

    #[test]
    fn test_safety_engine_rate_limit() {
        let engine = SafetyEngine::new();
        let constraints = vec![SafetyConstraint::RateLimit {
            action: "TestAction".to_string(),
            max_count: 3,
            window_secs: 3600,
        }];

        let params = serde_json::json!({});

        // First 3 calls should be allowed.
        for _ in 0..3 {
            let result = engine.check("TestAction", &params, &constraints);
            assert_eq!(result, SafetyResult::Allowed);
        }

        // 4th call should be rate limited.
        let result = engine.check("TestAction", &params, &constraints);
        match result {
            SafetyResult::Rejected { reason } => {
                assert!(reason.contains("Rate limit"));
                assert!(reason.contains("TestAction"));
            }
            other => panic!("Expected Rejected (rate limit), got {:?}", other),
        }
    }

    // -----------------------------------------------------------------------
    // Action Definition Builder Test
    // -----------------------------------------------------------------------

    #[test]
    fn test_action_definition_builder() {
        let action = ActionDefinition::new("TestAction")
            .description("A test action")
            .param("name", ParamType::String, true, "The name")
            .param("value", ParamType::Float, false, "The value")
            .param("tags", ParamType::StringArray, false, "Tags");

        assert_eq!(action.name, "TestAction");
        assert_eq!(action.description, "A test action");
        assert_eq!(action.params.len(), 3);
        assert_eq!(action.params[0].name, "name");
        assert_eq!(action.params[0].param_type, ParamType::String);
        assert!(action.params[0].required);
        assert_eq!(action.params[1].param_type, ParamType::Float);
        assert!(!action.params[1].required);
        assert_eq!(action.params[2].param_type, ParamType::StringArray);
    }

    // -----------------------------------------------------------------------
    // Strategy Preferences Test
    // -----------------------------------------------------------------------

    #[test]
    fn test_strategy_preferences_parsing() {
        let prefs = vec![
            StrategyPreference {
                task_pattern: "classify.*".to_string(),
                strategy: Strategy::Trm("model-a".to_string()),
                trm_model: Some("model-a".to_string()),
                rationale: "Classification works well with TRM.".to_string(),
            },
            StrategyPreference {
                task_pattern: "analyze.*".to_string(),
                strategy: Strategy::Rlm,
                trm_model: None,
                rationale: "Analysis needs reasoning.".to_string(),
            },
            StrategyPreference {
                task_pattern: "monitor.*".to_string(),
                strategy: Strategy::Hybrid {
                    triage: "model-a".to_string(),
                    threshold: 0.8,
                },
                trm_model: Some("model-a".to_string()),
                rationale: "Monitoring uses hybrid approach.".to_string(),
            },
        ];

        // Serialize and deserialize to verify round-trip.
        let json = serde_json::to_string(&prefs).unwrap();
        let deserialized: Vec<StrategyPreference> = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.len(), 3);
        assert_eq!(deserialized[0].task_pattern, "classify.*");
        match &deserialized[0].strategy {
            Strategy::Trm(name) => assert_eq!(name, "model-a"),
            other => panic!("Expected Trm, got {:?}", other),
        }
        match &deserialized[2].strategy {
            Strategy::Hybrid { triage, threshold } => {
                assert_eq!(triage, "model-a");
                assert!((threshold - 0.8).abs() < f64::EPSILON);
            }
            other => panic!("Expected Hybrid, got {:?}", other),
        }
    }

    // -----------------------------------------------------------------------
    // Template Plugin Test
    // -----------------------------------------------------------------------

    #[test]
    fn test_template_plugin_loads() {
        let plugin = MyDomainPlugin::new();
        assert_eq!(plugin.name(), "my-domain");
        assert_eq!(plugin.version(), semver::Version::new(0, 1, 0));
        assert!(!plugin.description().is_empty());

        // Should have 1 adapter, 1 strategy, 1 action, 1 constraint.
        assert_eq!(plugin.ingest_adapters().len(), 1);
        assert_eq!(plugin.strategy_preferences().len(), 1);
        assert_eq!(plugin.action_extensions().len(), 1);
        assert_eq!(plugin.safety_constraints().len(), 1);
        assert!(!plugin.system_prompt_extension().is_empty());

        // Can register in a registry.
        let mut registry = PluginRegistry::new();
        assert!(registry.register(Box::new(plugin)).is_ok());
        assert!(registry.get("my-domain").is_some());
    }
}
