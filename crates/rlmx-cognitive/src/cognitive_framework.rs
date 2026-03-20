//! Unified cognitive framework wrapping SONA + DagOptimizer + NervousSystem.
//!
//! When the `ruvnet-phase3` feature is enabled, delegates meta-learning to
//! `cognitum-rs` for unified cognitive orchestration. Otherwise, provides a
//! standalone implementation that detects stale patterns and schedules
//! retraining via internal heuristics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::dag::DagOptimizer;
use crate::sona::Sona;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum CognitiveError {
    #[error("no patterns available for analysis")]
    NoPatternsAvailable,
    #[error("retraining already in progress")]
    RetrainingInProgress,
    #[error("framework error: {0}")]
    FrameworkError(String),
}

pub type Result<T> = std::result::Result<T, CognitiveError>;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A pattern flagged as stale and candidate for retraining.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalePattern {
    pub pattern_id: Uuid,
    /// How long since last access, in seconds.
    pub age_secs: f64,
    /// Quality score at last observation.
    pub last_quality: f64,
    /// Reason the pattern was flagged stale.
    pub reason: String,
}

/// Summary statistics for the cognitive framework.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CognitiveStats {
    pub total_patterns: usize,
    pub stale_patterns: usize,
    pub avg_pattern_age_secs: f64,
    pub avg_quality: f64,
    pub retraining_cycles: usize,
    pub last_scan: Option<DateTime<Utc>>,
}

/// Configuration for staleness detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalenessConfig {
    /// Patterns older than this many seconds are candidates for staleness.
    pub max_age_secs: f64,
    /// Patterns with quality below this threshold are candidates.
    pub min_quality: f64,
    /// Patterns unused for this many seconds are candidates.
    pub unused_threshold_secs: f64,
}

impl Default for StalenessConfig {
    fn default() -> Self {
        Self {
            max_age_secs: 86_400.0 * 7.0, // 7 days
            min_quality: 0.3,
            unused_threshold_secs: 86_400.0 * 3.0, // 3 days
        }
    }
}

// ---------------------------------------------------------------------------
// Cognitive Framework
// ---------------------------------------------------------------------------

/// Manages the lifecycle of cognitive patterns by coordinating SONA's pattern
/// bank with the DAG optimizer's strategy records. Identifies stale patterns
/// and schedules retraining to keep the system's self-learning current.
pub struct CognitiveFramework {
    pub sona: Sona,
    pub dag_optimizer: DagOptimizer,
    pub staleness_config: StalenessConfig,
    pub retraining_cycles: usize,
    pub last_scan: Option<DateTime<Utc>>,
    cached_stale: Vec<StalePattern>,
}

impl CognitiveFramework {
    /// Create a new framework wrapping the given SONA and DagOptimizer.
    pub fn new(sona: Sona, dag_optimizer: DagOptimizer) -> Self {
        Self {
            sona,
            dag_optimizer,
            staleness_config: StalenessConfig::default(),
            retraining_cycles: 0,
            last_scan: None,
            cached_stale: Vec::new(),
        }
    }

    /// Create a framework with default-initialized components.
    pub fn with_defaults() -> Self {
        Self::new(Sona::new(), DagOptimizer::new())
    }

    /// Scan for stale patterns in the SONA pattern bank.
    ///
    /// When `ruvnet-phase3` is enabled, uses cognitum-rs's meta-learning
    /// engine for sophisticated staleness detection. Otherwise, applies
    /// simple age + quality thresholds.
    pub fn detect_stale(&mut self) -> Vec<StalePattern> {
        let now = Utc::now();
        self.last_scan = Some(now);

        #[cfg(feature = "ruvnet-phase3")]
        {
            self.detect_stale_cognitum(now)
        }

        #[cfg(not(feature = "ruvnet-phase3"))]
        {
            self.detect_stale_heuristic(now)
        }
    }

    /// Heuristic staleness detection based on age and quality thresholds.
    #[cfg(not(feature = "ruvnet-phase3"))]
    fn detect_stale_heuristic(&mut self, now: DateTime<Utc>) -> Vec<StalePattern> {
        let config = &self.staleness_config;
        let mut stale = Vec::new();

        for pattern in &self.sona.pattern_bank.patterns {
            let age_secs = (now - pattern.timestamp).num_seconds() as f64;

            let mut reasons = Vec::new();

            if age_secs > config.max_age_secs {
                reasons.push(format!(
                    "age {:.0}s exceeds max {:.0}s",
                    age_secs, config.max_age_secs
                ));
            }

            if pattern.result_quality < config.min_quality {
                reasons.push(format!(
                    "quality {:.2} below threshold {:.2}",
                    pattern.result_quality, config.min_quality
                ));
            }

            if pattern.usage_count == 0 && age_secs > config.unused_threshold_secs {
                reasons.push(format!(
                    "unused for {:.0}s (threshold {:.0}s)",
                    age_secs, config.unused_threshold_secs
                ));
            }

            if !reasons.is_empty() {
                stale.push(StalePattern {
                    pattern_id: pattern.id,
                    age_secs,
                    last_quality: pattern.result_quality,
                    reason: reasons.join("; "),
                });
            }
        }

        self.cached_stale = stale.clone();
        stale
    }

    /// Cognitum-rs backed staleness detection with meta-learning.
    #[cfg(feature = "ruvnet-phase3")]
    fn detect_stale_cognitum(&mut self, now: DateTime<Utc>) -> Vec<StalePattern> {
        use cognitum_rs::MetaLearner;

        let learner = MetaLearner::default();

        let pattern_data: Vec<(String, f64, f64)> = self
            .sona
            .pattern_bank
            .patterns
            .iter()
            .map(|p| {
                let age = (now - p.timestamp).num_seconds() as f64;
                (p.id.to_string(), p.result_quality, age)
            })
            .collect();

        let stale_ids = learner
            .detect_stale_patterns(&pattern_data)
            .unwrap_or_default();

        let stale: Vec<StalePattern> = self
            .sona
            .pattern_bank
            .patterns
            .iter()
            .filter(|p| stale_ids.contains(&p.id.to_string()))
            .map(|p| {
                let age_secs = (now - p.timestamp).num_seconds() as f64;
                StalePattern {
                    pattern_id: p.id,
                    age_secs,
                    last_quality: p.result_quality,
                    reason: "flagged by cognitum-rs meta-learner".to_string(),
                }
            })
            .collect();

        self.cached_stale = stale.clone();
        stale
    }

    /// Schedule retraining for stale patterns. Increments the retraining
    /// counter and evicts patterns below the quality threshold.
    pub fn schedule_retraining(&mut self) -> Result<usize> {
        if self.cached_stale.is_empty() {
            return Err(CognitiveError::NoPatternsAvailable);
        }

        let stale_ids: Vec<Uuid> = self.cached_stale.iter().map(|s| s.pattern_id).collect();
        let evicted = stale_ids.len();

        // Remove stale patterns from SONA's pattern bank.
        self.sona
            .pattern_bank
            .patterns
            .retain(|p| !stale_ids.contains(&p.id));
        self.sona
            .pattern_bank
            .embeddings
            .truncate(self.sona.pattern_bank.patterns.len());

        self.retraining_cycles += 1;
        self.cached_stale.clear();

        tracing::info!(
            evicted = evicted,
            cycle = self.retraining_cycles,
            "cognitive framework: scheduled retraining, evicted stale patterns"
        );

        Ok(evicted)
    }

    /// Return summary statistics.
    pub fn stats(&self) -> CognitiveStats {
        let total = self.sona.pattern_bank.patterns.len();
        let avg_quality = if total == 0 {
            0.0
        } else {
            self.sona
                .pattern_bank
                .patterns
                .iter()
                .map(|p| p.result_quality)
                .sum::<f64>()
                / total as f64
        };
        let avg_age = if total == 0 {
            0.0
        } else {
            let now = Utc::now();
            self.sona
                .pattern_bank
                .patterns
                .iter()
                .map(|p| (now - p.timestamp).num_seconds() as f64)
                .sum::<f64>()
                / total as f64
        };

        CognitiveStats {
            total_patterns: total,
            stale_patterns: self.cached_stale.len(),
            avg_pattern_age_secs: avg_age,
            avg_quality,
            retraining_cycles: self.retraining_cycles,
            last_scan: self.last_scan,
        }
    }
}

impl Default for CognitiveFramework {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl std::fmt::Debug for CognitiveFramework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CognitiveFramework")
            .field("patterns", &self.sona.pattern_bank.patterns.len())
            .field("stale_cached", &self.cached_stale.len())
            .field("retraining_cycles", &self.retraining_cycles)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_defaults() {
        let fw = CognitiveFramework::with_defaults();
        let stats = fw.stats();
        assert_eq!(stats.total_patterns, 0);
        assert_eq!(stats.stale_patterns, 0);
        assert_eq!(stats.retraining_cycles, 0);
    }

    #[test]
    fn test_detect_stale_empty() {
        let mut fw = CognitiveFramework::with_defaults();
        let stale = fw.detect_stale();
        assert!(stale.is_empty());
    }

    #[test]
    fn test_detect_stale_low_quality() {
        let mut fw = CognitiveFramework::with_defaults();
        // Record a low-quality pattern.
        fw.sona
            .record_pattern("bad query", vec!["fail".into()], 0.1);

        // With default config, quality 0.1 < 0.3 threshold => stale.
        let stale = fw.detect_stale();
        assert_eq!(stale.len(), 1);
        assert!(stale[0].reason.contains("quality"));
    }

    #[test]
    fn test_schedule_retraining_evicts() {
        let mut fw = CognitiveFramework::with_defaults();
        fw.sona.record_pattern("q1", vec!["a".into()], 0.1);
        fw.sona.record_pattern("q2", vec!["b".into()], 0.9);

        fw.detect_stale();
        let before = fw.sona.pattern_bank.patterns.len();
        let evicted = fw.schedule_retraining().unwrap();

        assert!(evicted > 0);
        assert!(fw.sona.pattern_bank.patterns.len() < before);
        assert_eq!(fw.retraining_cycles, 1);
    }

    #[test]
    fn test_schedule_retraining_no_stale_err() {
        let mut fw = CognitiveFramework::with_defaults();
        let err = fw.schedule_retraining().unwrap_err();
        assert!(matches!(err, CognitiveError::NoPatternsAvailable));
    }

    #[test]
    fn test_stats_reflect_patterns() {
        let mut fw = CognitiveFramework::with_defaults();
        fw.sona.record_pattern("q1", vec!["a".into()], 0.8);
        fw.sona.record_pattern("q2", vec!["b".into()], 0.6);

        let stats = fw.stats();
        assert_eq!(stats.total_patterns, 2);
        assert!((stats.avg_quality - 0.7).abs() < 0.01);
    }
}
