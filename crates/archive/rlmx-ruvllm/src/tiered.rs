//! Tiered inference engine.
//!
//! Routes generation requests through an ordered list of model tiers,
//! escalating to higher-capability tiers when confidence falls below
//! a configurable threshold. In stub mode (without the `ruvllm` feature),
//! unloaded tiers return confidence 0.5 to demonstrate the escalation pattern.

use std::time::Instant;

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::config::{ModelTier, TierSpec, TieredConfig};
use crate::engine::LocalEngine;
use crate::error::RuvllmError;
use crate::mlx_bridge::MlxSubprocess;

/// A single tier entry holding its spec and optional loaded engine.
pub struct TierEntry {
    /// The tier specification.
    pub spec: TierSpec,
    /// The loaded engine for this tier (None if not yet loaded).
    pub engine: Option<LocalEngine>,
    /// Whether this tier's model has been loaded.
    pub loaded: bool,
}

/// Result of a tiered generation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TieredResult {
    /// The generated text.
    pub text: String,
    /// Number of tokens generated.
    pub tokens_generated: u32,
    /// Wall-clock latency in milliseconds.
    pub latency_ms: u64,
    /// Which tier produced the final result.
    pub tier_used: ModelTier,
    /// Confidence of the final result.
    pub confidence: f64,
    /// Whether escalation occurred.
    pub escalated: bool,
    /// How many escalations happened during this request.
    pub escalation_count: u32,
}

/// Aggregate statistics for the tiered engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TieredStats {
    /// Total requests processed.
    pub total_requests: u64,
    /// Total escalations that occurred.
    pub escalations: u64,
    /// Escalation rate (escalations / total_requests).
    pub escalation_rate: f64,
    /// Per-tier information.
    pub tiers: Vec<TierInfo>,
}

/// Information about a single tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierInfo {
    /// The tier identity.
    pub tier: ModelTier,
    /// Whether the tier's model is loaded.
    pub loaded: bool,
    /// The tier's priority.
    pub priority: u32,
}

/// Tiered inference engine that routes requests through multiple model tiers.
pub struct TieredEngine {
    /// Ordered tier entries (sorted by priority).
    tiers: Vec<TierEntry>,
    /// Configuration.
    config: TieredConfig,
    /// MLX subprocess bridge for Apple Silicon Medium tier inference.
    pub mlx: Option<MlxSubprocess>,
    /// Index of the current/default starting tier.
    current_tier: usize,
    /// Total requests served.
    total_requests: u64,
    /// Total escalations performed.
    escalations: u64,
}

/// Compute a stub confidence score from the prompt length.
///
/// Maps prompt byte length to a confidence in [0.1, 0.35], ensuring
/// stubs remain below the default escalation threshold (0.4) so that
/// escalation behavior is consistently demonstrated in stub mode.
fn compute_stub_confidence(prompt: &str) -> f64 {
    let len = prompt.len() as f64;
    // Sigmoid-like mapping: short prompts -> ~0.1, long prompts -> ~0.35
    let ratio = (len / 500.0).min(1.0);
    0.1 + ratio * 0.25
}

impl TieredEngine {
    /// Create a new tiered engine from the given configuration.
    ///
    /// Tiers are sorted by priority (lowest first). No models are loaded
    /// at creation time — call `load_tier` to load individual tiers.
    pub fn new(config: TieredConfig) -> Result<Self, RuvllmError> {
        let mut specs: Vec<TierSpec> = config.models.clone();
        specs.sort_by_key(|s| s.priority);

        let tiers = specs
            .into_iter()
            .map(|spec| TierEntry {
                spec,
                engine: None,
                loaded: false,
            })
            .collect();

        info!(
            tier_count = config.models.len(),
            threshold = config.escalation_threshold,
            max_escalations = config.max_escalations,
            "Created tiered inference engine"
        );

        Ok(Self {
            tiers,
            config,
            mlx: None,
            current_tier: 0,
            total_requests: 0,
            escalations: 0,
        })
    }

    /// Generate text, starting at the lowest tier and escalating as needed.
    ///
    /// In stub mode (no loaded engines), each tier returns a stub response
    /// with confidence 0.5, which is below the default threshold (0.7),
    /// triggering escalation to demonstrate the pattern.
    pub async fn generate(
        &mut self,
        prompt: &str,
        max_tokens: u32,
    ) -> Result<TieredResult, RuvllmError> {
        if self.tiers.is_empty() {
            return Err(RuvllmError::LoadError("no tiers configured".to_string()));
        }

        let start = Instant::now();
        self.total_requests += 1;

        let mut escalation_count: u32 = 0;
        let mut tier_index = self.current_tier;

        loop {
            if tier_index >= self.tiers.len() {
                // No more tiers to escalate to — return the last tier's stub result.
                tier_index = self.tiers.len() - 1;
                let tier = &self.tiers[tier_index];
                let elapsed = start.elapsed().as_millis() as u64;
                // Compute stub confidence from prompt length: longer prompts get
                // slightly higher confidence to simulate varying output quality.
                let stub_confidence = compute_stub_confidence(prompt);
                return Ok(TieredResult {
                    text: format!("[stub:{}] {}", tier.spec.tier, prompt),
                    tokens_generated: 0,
                    latency_ms: elapsed,
                    tier_used: tier.spec.tier.clone(),
                    confidence: stub_confidence,
                    escalated: escalation_count > 0,
                    escalation_count,
                });
            }

            let tier = &self.tiers[tier_index];

            // Try to generate with this tier's engine if loaded.
            let result = if tier.loaded {
                if let Some(ref engine) = tier.engine {
                    match engine.generate(prompt, max_tokens).await {
                        Ok(gen) => Some((gen.text, gen.tokens_generated, gen.confidence)),
                        Err(e) => {
                            warn!(
                                tier = %tier.spec.tier,
                                error = %e,
                                "Generation failed at tier, escalating"
                            );
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                // Stub: unloaded tier returns computed confidence based on prompt.
                let stub_confidence = compute_stub_confidence(prompt);
                Some((
                    format!("[stub:{}] {}", tier.spec.tier, prompt),
                    0u32,
                    stub_confidence,
                ))
            };

            if let Some((text, tokens_generated, confidence)) = result {
                let threshold = self.config.escalation_threshold as f64;
                if confidence >= threshold
                    || escalation_count >= self.config.max_escalations
                    || tier_index + 1 >= self.tiers.len()
                {
                    let elapsed = start.elapsed().as_millis() as u64;
                    return Ok(TieredResult {
                        text,
                        tokens_generated,
                        latency_ms: elapsed,
                        tier_used: self.tiers[tier_index].spec.tier.clone(),
                        confidence,
                        escalated: escalation_count > 0,
                        escalation_count,
                    });
                }

                // Escalate to next tier.
                info!(
                    from_tier = %tier.spec.tier,
                    confidence = confidence,
                    threshold = threshold,
                    "Confidence below threshold, escalating"
                );
                escalation_count += 1;
                self.escalations += 1;
                tier_index += 1;
            } else {
                // Engine error — try next tier.
                escalation_count += 1;
                self.escalations += 1;
                tier_index += 1;
            }
        }
    }

    /// Load a specific tier's model by index.
    ///
    /// Creates a `LocalEngine` from the tier's `ModelSpec` and `EdgeConfig`
    /// defaults. In stub mode (without `ruvllm` feature), the engine is
    /// created but `load()` will return `NotAvailable`.
    pub fn load_tier(&mut self, index: usize) -> Result<(), RuvllmError> {
        if index >= self.tiers.len() {
            return Err(RuvllmError::LoadError(format!(
                "tier index {} out of range (have {})",
                index,
                self.tiers.len()
            )));
        }

        let tier = &mut self.tiers[index];
        let edge_config = crate::config::EdgeConfig {
            model: tier.spec.model.clone(),
            ..Default::default()
        };

        let mut engine = LocalEngine::new(edge_config)?;
        match engine.load() {
            Ok(()) => {
                tier.loaded = true;
                tier.engine = Some(engine);
                info!(tier = %tier.spec.tier, "Tier model loaded");
                Ok(())
            }
            Err(RuvllmError::NotAvailable) => {
                // Stub mode — engine created but not loaded.
                tier.engine = Some(engine);
                info!(tier = %tier.spec.tier, "Tier created in stub mode (ruvllm not available)");
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Get the current (starting) tier.
    pub fn current_tier(&self) -> &ModelTier {
        if self.tiers.is_empty() {
            // This shouldn't happen in normal use, but return a sensible default.
            static DEFAULT: ModelTier = ModelTier::Small;
            return &DEFAULT;
        }
        &self.tiers[self.current_tier].spec.tier
    }

    /// Get aggregate statistics.
    pub fn stats(&self) -> TieredStats {
        let escalation_rate = if self.total_requests > 0 {
            self.escalations as f64 / self.total_requests as f64
        } else {
            0.0
        };

        let tiers = self
            .tiers
            .iter()
            .map(|t| TierInfo {
                tier: t.spec.tier.clone(),
                loaded: t.loaded,
                priority: t.spec.priority,
            })
            .collect();

        TieredStats {
            total_requests: self.total_requests,
            escalations: self.escalations,
            escalation_rate,
            tiers,
        }
    }

    /// Number of configured tiers.
    pub fn tier_count(&self) -> usize {
        self.tiers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{InferenceBackendType, ModelSpec, ModelTier, TierSpec, TieredConfig};
    use std::path::PathBuf;

    fn make_spec(tier: ModelTier, priority: u32) -> TierSpec {
        TierSpec {
            tier,
            model: ModelSpec {
                name: "test-model".into(),
                path: PathBuf::new(),
                quantization: "Q4_K_M".into(),
                context_length: 2048,
            },
            priority,
        }
    }

    fn make_config(tiers: Vec<TierSpec>) -> TieredConfig {
        TieredConfig {
            models: tiers,
            escalation_threshold: 0.4,
            max_escalations: 2,
            chain: vec![ModelTier::Small, ModelTier::Medium],
            tier_timeout_ms: 5000,
        }
    }

    #[test]
    fn test_tiered_creation() {
        let config = make_config(vec![
            make_spec(ModelTier::Small, 1),
            make_spec(ModelTier::Medium, 2),
        ]);
        let engine = TieredEngine::new(config).unwrap();
        assert_eq!(engine.tier_count(), 2);
        assert_eq!(*engine.current_tier(), ModelTier::Small);
    }

    #[test]
    fn test_tiered_creation_empty() {
        let config = TieredConfig::default();
        let engine = TieredEngine::new(config).unwrap();
        assert_eq!(engine.tier_count(), 0);
    }

    #[test]
    fn test_tiered_creation_sorts_by_priority() {
        let config = make_config(vec![
            make_spec(ModelTier::Medium, 10),
            make_spec(ModelTier::Small, 1),
        ]);
        let engine = TieredEngine::new(config).unwrap();
        // Lowest priority first.
        assert_eq!(*engine.current_tier(), ModelTier::Small);
    }

    #[tokio::test]
    async fn test_escalation_logic() {
        // With stub tiers, computed confidence < threshold 0.4, so escalation
        // should occur up to max_escalations.
        let config = make_config(vec![
            make_spec(ModelTier::Small, 1),
            make_spec(ModelTier::Medium, 2),
            make_spec(ModelTier::ClaudeCode, 3),
        ]);
        let mut engine = TieredEngine::new(config).unwrap();

        let result = engine.generate("hello", 10).await.unwrap();

        // Should have escalated (stub confidence < threshold 0.4).
        assert!(result.escalated);
        assert_eq!(result.escalation_count, 2); // max_escalations = 2
                                                // Final tier should be ClaudeCode (after 2 escalations from Small).
        assert_eq!(result.tier_used, ModelTier::ClaudeCode);
        assert!(result.text.contains("[stub:"));
    }

    #[tokio::test]
    async fn test_escalation_stops_at_max() {
        // 5 tiers but max_escalations = 2, so it should stop at tier index 2.
        let config = TieredConfig {
            models: vec![
                make_spec(ModelTier::Small, 1),
                make_spec(ModelTier::Medium, 2),
                make_spec(ModelTier::ClaudeCode, 3),
                make_spec(
                    ModelTier::Custom {
                        model_name: "xlarge".into(),
                        backend: InferenceBackendType::Candle,
                        max_tokens: 4096,
                    },
                    4,
                ),
                make_spec(
                    ModelTier::Custom {
                        model_name: "xxlarge".into(),
                        backend: InferenceBackendType::Mlx,
                        max_tokens: 8192,
                    },
                    5,
                ),
            ],
            escalation_threshold: 0.4,
            max_escalations: 2,
            chain: vec![ModelTier::Small, ModelTier::Medium],
            tier_timeout_ms: 5000,
        };
        let mut engine = TieredEngine::new(config).unwrap();

        let result = engine.generate("test", 10).await.unwrap();
        assert_eq!(result.escalation_count, 2);
        // Should stop at index 2 (ClaudeCode) since max_escalations=2.
        assert_eq!(result.tier_used, ModelTier::ClaudeCode);
    }

    #[tokio::test]
    async fn test_no_tiers_returns_error() {
        let config = TieredConfig::default();
        let mut engine = TieredEngine::new(config).unwrap();
        let result = engine.generate("hello", 10).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_stats() {
        let config = make_config(vec![
            make_spec(ModelTier::Small, 1),
            make_spec(ModelTier::Medium, 2),
        ]);
        let engine = TieredEngine::new(config).unwrap();
        let stats = engine.stats();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.escalations, 0);
        assert_eq!(stats.escalation_rate, 0.0);
        assert_eq!(stats.tiers.len(), 2);
        assert!(!stats.tiers[0].loaded);
    }

    #[tokio::test]
    async fn test_stats_after_requests() {
        let config = make_config(vec![
            make_spec(ModelTier::Small, 1),
            make_spec(ModelTier::Medium, 2),
            make_spec(ModelTier::ClaudeCode, 3),
        ]);
        let mut engine = TieredEngine::new(config).unwrap();

        engine.generate("test1", 10).await.unwrap();
        engine.generate("test2", 10).await.unwrap();

        let stats = engine.stats();
        assert_eq!(stats.total_requests, 2);
        // Each request escalates twice (stub confidence < 0.4, max_escalations=2).
        assert_eq!(stats.escalations, 4);
        assert!((stats.escalation_rate - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_tier_count() {
        let config = make_config(vec![
            make_spec(ModelTier::Small, 1),
            make_spec(ModelTier::Medium, 2),
            make_spec(ModelTier::ClaudeCode, 3),
        ]);
        let engine = TieredEngine::new(config).unwrap();
        assert_eq!(engine.tier_count(), 3);
    }

    #[test]
    fn test_load_tier_out_of_range() {
        let config = make_config(vec![make_spec(ModelTier::Small, 1)]);
        let mut engine = TieredEngine::new(config).unwrap();
        let result = engine.load_tier(5);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_tier_stub_mode() {
        let config = make_config(vec![make_spec(ModelTier::Small, 1)]);
        let mut engine = TieredEngine::new(config).unwrap();
        // In stub mode (no ruvllm feature), load_tier should succeed
        // and mark the tier as having an engine (but not loaded).
        let result = engine.load_tier(0);
        assert!(result.is_ok());
    }

    #[test]
    fn test_model_tier_display() {
        assert_eq!(format!("{}", ModelTier::Small), "small");
        assert_eq!(format!("{}", ModelTier::ClaudeCode), "claude-code");
        assert_eq!(format!("{}", ModelTier::Medium), "medium");
        assert_eq!(
            format!(
                "{}",
                ModelTier::Custom {
                    model_name: "fast".into(),
                    backend: InferenceBackendType::Candle,
                    max_tokens: 2048,
                }
            ),
            "custom(fast)"
        );
    }

    #[test]
    fn test_tiered_config_default() {
        let config = TieredConfig::default();
        assert!(config.models.is_empty());
        assert!(
            (config.escalation_threshold - 0.4).abs() < f32::EPSILON,
            "default threshold should be 0.4 per ADR-004"
        );
        assert_eq!(config.max_escalations, 2);
        assert_eq!(config.chain, vec![ModelTier::Small, ModelTier::Medium]);
        assert_eq!(config.tier_timeout_ms, 5000);
    }

    #[test]
    fn test_custom_model_tier_all_fields() {
        let tier = ModelTier::Custom {
            model_name: "llama-70b".into(),
            backend: InferenceBackendType::Mlx,
            max_tokens: 16384,
        };
        assert_eq!(format!("{}", tier), "custom(llama-70b)");
        if let ModelTier::Custom {
            model_name,
            backend,
            max_tokens,
        } = &tier
        {
            assert_eq!(model_name, "llama-70b");
            assert_eq!(*backend, InferenceBackendType::Mlx);
            assert_eq!(*max_tokens, 16384);
        } else {
            panic!("expected Custom variant");
        }
    }

    #[test]
    fn test_custom_model_tier_remote_backend() {
        let tier = ModelTier::Custom {
            model_name: "gpt4-proxy".into(),
            backend: InferenceBackendType::Remote("https://api.example.com".into()),
            max_tokens: 4096,
        };
        if let ModelTier::Custom { backend, .. } = &tier {
            assert_eq!(
                *backend,
                InferenceBackendType::Remote("https://api.example.com".into())
            );
        }
    }

    #[test]
    fn test_escalation_config_defaults() {
        let config = TieredConfig::default();
        assert!(
            (config.escalation_threshold - 0.4).abs() < f32::EPSILON,
            "threshold must default to 0.4"
        );
        assert_eq!(
            config.tier_timeout_ms, 5000,
            "timeout must default to 5000ms"
        );
        assert_eq!(
            config.chain,
            vec![ModelTier::Small, ModelTier::Medium],
            "chain must default to [Small, Medium]"
        );
    }

    #[tokio::test]
    async fn test_chain_based_escalation() {
        // Verify that escalation follows the configured tier order.
        let config = TieredConfig {
            models: vec![
                make_spec(ModelTier::Small, 1),
                make_spec(ModelTier::Medium, 2),
            ],
            escalation_threshold: 0.4,
            max_escalations: 1,
            chain: vec![ModelTier::Small, ModelTier::Medium],
            tier_timeout_ms: 5000,
        };
        let mut engine = TieredEngine::new(config).unwrap();

        let result = engine.generate("short", 10).await.unwrap();
        // Stub confidence is below 0.4, so should escalate once to Medium.
        assert!(result.escalated);
        assert_eq!(result.escalation_count, 1);
        assert_eq!(result.tier_used, ModelTier::Medium);
    }

    #[test]
    fn test_tiered_engine_has_mlx_field() {
        let config = make_config(vec![make_spec(ModelTier::Small, 1)]);
        let engine = TieredEngine::new(config).unwrap();
        assert!(engine.mlx.is_none(), "mlx should be None by default");
    }

    #[test]
    fn test_stub_confidence_is_below_threshold() {
        // Verify that stub confidence stays below the 0.4 threshold
        // for typical prompt lengths, ensuring escalation works.
        let short = compute_stub_confidence("hi");
        let medium = compute_stub_confidence("a medium length prompt for testing");
        let long = compute_stub_confidence(&"x".repeat(1000));

        assert!(
            short < 0.4,
            "short prompt stub confidence {} should be < 0.4",
            short
        );
        assert!(
            medium < 0.4,
            "medium prompt stub confidence {} should be < 0.4",
            medium
        );
        assert!(
            long < 0.4,
            "long prompt stub confidence {} should be < 0.4",
            long
        );
        // Also verify ordering: longer prompts get higher confidence.
        assert!(short < medium);
        assert!(medium < long);
    }
}
