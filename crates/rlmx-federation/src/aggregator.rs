//! Federated pattern aggregation: merges contributions from 1000+ users.
//!
//! Implements the cloud-side aggregation step (ADR-023 Step 3):
//! - Top-10K pattern selection per domain, ranked by outcome quality.
//! - Aggregated LoRA delta computation (weighted average).
//! - Minimum 1000-user aggregation threshold enforced.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

use rlmx_cognitive::sona::LoraDelta;
use rlmx_cognitive::voice_patterns::AnonymizedPattern;
use rlmx_kernel::LifeDomain;

use crate::contribution::Contribution;
use crate::error::{FederationError, Result};

/// Configuration for the aggregation engine.
#[derive(Debug, Clone)]
pub struct AggregatorConfig {
    /// Minimum number of contributors required before aggregation.
    pub min_contributors: usize,
    /// Maximum number of patterns to keep per domain after aggregation.
    pub top_k_patterns: usize,
}

impl Default for AggregatorConfig {
    fn default() -> Self {
        Self {
            min_contributors: 1000,
            top_k_patterns: 10_000,
        }
    }
}

/// Result of aggregating contributions for a single domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedModel {
    /// Unique identifier for this aggregated model.
    pub id: Uuid,
    /// The domain these patterns belong to.
    pub domain: LifeDomain,
    /// Number of distinct contributors.
    pub contributor_count: usize,
    /// Top-K patterns ranked by quality.
    pub patterns: Vec<AnonymizedPattern>,
    /// Aggregated LoRA delta (weighted average across contributors).
    pub lora_delta: Option<LoraDelta>,
    /// Updated router weights (stub: domain-level confidence).
    pub router_confidence: f64,
    /// When this model was created.
    pub created_at: DateTime<Utc>,
}

/// Aggregates anonymized pattern contributions across users.
#[derive(Debug)]
pub struct FederatedAggregator {
    config: AggregatorConfig,
}

impl FederatedAggregator {
    /// Create a new aggregator with the given config.
    pub fn new(config: AggregatorConfig) -> Self {
        Self { config }
    }

    /// Aggregate contributions for a given domain.
    ///
    /// Enforces the minimum contributor threshold before proceeding.
    /// Returns the aggregated model with top-K patterns and merged LoRA delta.
    pub fn aggregate(
        &self,
        domain: LifeDomain,
        contributions: &[Contribution],
    ) -> Result<AggregatedModel> {
        // Filter contributions for this domain.
        let domain_contributions: Vec<&Contribution> = contributions
            .iter()
            .filter(|c| c.domain == domain)
            .collect();

        if domain_contributions.is_empty() {
            return Err(FederationError::NoDomainContributions(domain));
        }

        // Count distinct contributors.
        let unique_contributors: std::collections::HashSet<&str> = domain_contributions
            .iter()
            .map(|c| c.user_pseudonym.as_str())
            .collect();

        let contributor_count = unique_contributors.len();

        if contributor_count < self.config.min_contributors {
            warn!(
                domain = ?domain,
                contributor_count,
                threshold = self.config.min_contributors,
                "aggregation threshold not met"
            );
            return Err(FederationError::AggregationThresholdNotMet {
                required: self.config.min_contributors,
                actual: contributor_count,
            });
        }

        info!(
            domain = ?domain,
            contributor_count,
            "aggregating federated contributions"
        );

        // Collect all patterns and select top-K by quality.
        let top_patterns = self.select_top_patterns(&domain_contributions);

        // Compute aggregated LoRA delta.
        let aggregated_lora = self.aggregate_lora_deltas(&domain_contributions);

        // Compute average router confidence from contribution qualities.
        let avg_quality: f64 = domain_contributions
            .iter()
            .map(|c| c.aggregate_quality as f64)
            .sum::<f64>()
            / domain_contributions.len() as f64;

        debug!(
            domain = ?domain,
            pattern_count = top_patterns.len(),
            has_lora = aggregated_lora.is_some(),
            avg_quality,
            "aggregation complete"
        );

        Ok(AggregatedModel {
            id: Uuid::new_v4(),
            domain,
            contributor_count,
            patterns: top_patterns,
            lora_delta: aggregated_lora,
            router_confidence: avg_quality,
            created_at: Utc::now(),
        })
    }

    /// Aggregate all domains at once from a mixed set of contributions.
    ///
    /// Returns a map of domain to aggregated model. Domains that do not meet
    /// the threshold are omitted (not treated as errors).
    pub fn aggregate_all(
        &self,
        contributions: &[Contribution],
    ) -> HashMap<LifeDomain, AggregatedModel> {
        // Discover all domains present.
        let domains: std::collections::HashSet<LifeDomain> =
            contributions.iter().map(|c| c.domain).collect();

        let mut results = HashMap::new();
        for domain in domains {
            match self.aggregate(domain, contributions) {
                Ok(model) => {
                    results.insert(domain, model);
                }
                Err(e) => {
                    debug!(domain = ?domain, error = %e, "skipping domain in aggregate_all");
                }
            }
        }
        results
    }

    /// Select the top-K patterns by result quality across all contributions.
    ///
    /// Uses partial sort to avoid cloning/sorting all patterns when top_k << total.
    fn select_top_patterns(&self, contributions: &[&Contribution]) -> Vec<AnonymizedPattern> {
        let total: usize = contributions.iter().map(|c| c.patterns.len()).sum();
        let k = self.config.top_k_patterns;

        if total <= k {
            // All patterns fit — clone them all, no sorting needed.
            return contributions
                .iter()
                .flat_map(|c| c.patterns.clone())
                .collect();
        }

        // Collect references with quality scores, then select top-k by partial sort.
        let mut indexed: Vec<(f32, usize, usize)> = Vec::with_capacity(total);
        for (ci, c) in contributions.iter().enumerate() {
            for (pi, p) in c.patterns.iter().enumerate() {
                indexed.push((p.result_quality, ci, pi));
            }
        }

        // Partial sort: partition so the top-k are in positions [0..k].
        indexed.select_nth_unstable_by(k - 1, |a, b| {
            b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Clone only the top-k patterns.
        let mut result: Vec<AnonymizedPattern> = indexed[..k]
            .iter()
            .map(|&(_, ci, pi)| contributions[ci].patterns[pi].clone())
            .collect();
        result.sort_by(|a, b| {
            b.result_quality
                .partial_cmp(&a.result_quality)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        result
    }

    /// Compute a weighted average of LoRA deltas across contributions.
    ///
    /// Weights are the contribution's aggregate quality score.
    fn aggregate_lora_deltas(&self, contributions: &[&Contribution]) -> Option<LoraDelta> {
        let deltas_with_weights: Vec<(&LoraDelta, f32)> = contributions
            .iter()
            .filter_map(|c| c.lora_delta.as_ref().map(|d| (d, c.aggregate_quality)))
            .collect();

        if deltas_with_weights.is_empty() {
            return None;
        }

        // Use the first delta as a template for shape.
        let (template, _) = &deltas_with_weights[0];
        let rank = template.rank;
        let layer_name = template.layer_name.clone();
        let a_len = template.delta_a.len();
        let b_len = template.delta_b.len();

        // Only aggregate deltas with matching shapes.
        let compatible: Vec<(&LoraDelta, f32)> = deltas_with_weights
            .into_iter()
            .filter(|(d, _)| d.rank == rank && d.delta_a.len() == a_len && d.delta_b.len() == b_len)
            .collect();

        if compatible.is_empty() {
            return None;
        }

        let total_weight: f32 = compatible.iter().map(|(_, w)| w).sum();
        if total_weight <= 0.0 {
            return None;
        }

        let mut avg_a = vec![0.0f32; a_len];
        let mut avg_b = vec![0.0f32; b_len];

        for (delta, weight) in &compatible {
            let w = weight / total_weight;
            for (i, v) in delta.delta_a.iter().enumerate() {
                avg_a[i] += v * w;
            }
            for (i, v) in delta.delta_b.iter().enumerate() {
                avg_b[i] += v * w;
            }
        }

        Some(LoraDelta {
            layer_name,
            delta_a: avg_a,
            delta_b: avg_b,
            rank,
            applied_at: Utc::now(),
        })
    }
}

impl Default for FederatedAggregator {
    fn default() -> Self {
        Self::new(AggregatorConfig::default())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rlmx_cognitive::voice_patterns::Modality;

    fn make_contribution(domain: LifeDomain, pseudonym: &str, quality: f32) -> Contribution {
        let pattern = AnonymizedPattern {
            id: Uuid::new_v4(),
            sanitized_embedding: vec![0.1; 64],
            actions_taken: vec!["action".into()],
            result_quality: quality,
            emotion_bucket: None,
            noisy_urgency: 0.5,
            noisy_satisfaction: None,
            interaction_modality: Modality::Voice,
            timestamp: Utc::now(),
        };
        Contribution::new(pseudonym.into(), domain, vec![pattern], None)
    }

    fn make_many_contributions(domain: LifeDomain, count: usize) -> Vec<Contribution> {
        (0..count)
            .map(|i| make_contribution(domain, &format!("user-{i}"), 0.5 + (i as f32 * 0.0001)))
            .collect()
    }

    #[test]
    fn test_aggregate_threshold_not_met() {
        let agg = FederatedAggregator::default();
        let contribs = make_many_contributions(LifeDomain::Finance, 10);
        let result = agg.aggregate(LifeDomain::Finance, &contribs);
        assert!(matches!(
            result,
            Err(FederationError::AggregationThresholdNotMet { .. })
        ));
    }

    #[test]
    fn test_aggregate_no_domain_contributions() {
        let agg = FederatedAggregator::default();
        let contribs = make_many_contributions(LifeDomain::Finance, 10);
        let result = agg.aggregate(LifeDomain::Health, &contribs);
        assert!(matches!(
            result,
            Err(FederationError::NoDomainContributions(_))
        ));
    }

    #[test]
    fn test_aggregate_success_with_low_threshold() {
        let config = AggregatorConfig {
            min_contributors: 3,
            top_k_patterns: 5,
        };
        let agg = FederatedAggregator::new(config);
        let contribs = make_many_contributions(LifeDomain::Finance, 5);
        let model = agg.aggregate(LifeDomain::Finance, &contribs).unwrap();
        assert_eq!(model.domain, LifeDomain::Finance);
        assert_eq!(model.contributor_count, 5);
        assert_eq!(model.patterns.len(), 5); // 5 contributions, 1 pattern each, top_k=5
    }

    #[test]
    fn test_aggregate_top_k_truncation() {
        let config = AggregatorConfig {
            min_contributors: 2,
            top_k_patterns: 3,
        };
        let agg = FederatedAggregator::new(config);
        // 5 contributions, each with 1 pattern = 5 patterns total, top_k = 3.
        let contribs = make_many_contributions(LifeDomain::Health, 5);
        let model = agg.aggregate(LifeDomain::Health, &contribs).unwrap();
        assert_eq!(model.patterns.len(), 3);
        // Top-3 should be the highest quality.
        assert!(model.patterns[0].result_quality >= model.patterns[1].result_quality);
    }

    #[test]
    fn test_aggregate_lora_deltas() {
        let config = AggregatorConfig {
            min_contributors: 2,
            top_k_patterns: 100,
        };
        let agg = FederatedAggregator::new(config);

        let delta1 = LoraDelta {
            layer_name: "layer_0".into(),
            delta_a: vec![1.0, 2.0],
            delta_b: vec![0.5],
            rank: 1,
            applied_at: Utc::now(),
        };
        let delta2 = LoraDelta {
            layer_name: "layer_0".into(),
            delta_a: vec![3.0, 4.0],
            delta_b: vec![1.5],
            rank: 1,
            applied_at: Utc::now(),
        };

        let pattern = AnonymizedPattern {
            id: Uuid::new_v4(),
            sanitized_embedding: vec![0.1; 8],
            actions_taken: vec!["a".into()],
            result_quality: 0.8,
            emotion_bucket: None,
            noisy_urgency: 0.5,
            noisy_satisfaction: None,
            interaction_modality: Modality::Text,
            timestamp: Utc::now(),
        };

        let mut c1 = Contribution::new(
            "user-a".into(),
            LifeDomain::Finance,
            vec![pattern.clone()],
            Some(delta1),
        );
        c1.aggregate_quality = 0.8;
        let mut c2 = Contribution::new(
            "user-b".into(),
            LifeDomain::Finance,
            vec![pattern],
            Some(delta2),
        );
        c2.aggregate_quality = 0.8;

        let model = agg.aggregate(LifeDomain::Finance, &[c1, c2]).unwrap();
        let lora = model.lora_delta.unwrap();
        // Equal weights -> average of deltas.
        assert!((lora.delta_a[0] - 2.0).abs() < 0.01);
        assert!((lora.delta_a[1] - 3.0).abs() < 0.01);
        assert!((lora.delta_b[0] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_aggregate_all_mixed_domains() {
        let config = AggregatorConfig {
            min_contributors: 2,
            top_k_patterns: 100,
        };
        let agg = FederatedAggregator::new(config);

        let mut contribs = make_many_contributions(LifeDomain::Finance, 3);
        contribs.extend(make_many_contributions(LifeDomain::Health, 2));
        // Only 1 Legal contribution — won't meet threshold.
        contribs.push(make_contribution(LifeDomain::Legal, "legal-user", 0.9));

        let results = agg.aggregate_all(&contribs);
        assert!(results.contains_key(&LifeDomain::Finance));
        assert!(results.contains_key(&LifeDomain::Health));
        assert!(!results.contains_key(&LifeDomain::Legal));
    }
}
