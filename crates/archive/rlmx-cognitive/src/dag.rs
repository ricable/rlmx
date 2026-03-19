//! DAG Query Optimizer
//!
//! A self-learning query optimizer that records execution history and learns
//! optimal strategies (execution plan + attention mechanism) per query pattern.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// A single execution record for a query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub query_hash: String,
    pub query_pattern: String, // Simplified pattern for grouping
    pub strategy_used: String,
    pub attention_mechanism: String,
    pub latency_ms: f64,
    pub quality_score: f64,
    pub timestamp: DateTime<Utc>,
}

/// Aggregate statistics for a particular strategy within a query pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyStats {
    pub strategy: String,
    pub avg_latency_ms: f64,
    pub avg_quality: f64,
    pub sample_count: usize,
    pub best_attention: String,
}

// ---------------------------------------------------------------------------
// DagOptimizer
// ---------------------------------------------------------------------------

/// Self-learning DAG query optimizer.
#[derive(Debug, Clone, Default)]
pub struct DagOptimizer {
    /// Full execution history.
    pub execution_history: Vec<ExecutionRecord>,
    /// Learned strategy preferences per query pattern.
    pub strategy_cache: HashMap<String, StrategyStats>,
    /// Total queries processed.
    pub total_queries: usize,
}

impl DagOptimizer {
    /// Create a new, empty optimizer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an execution and update the strategy cache.
    pub fn record(&mut self, record: ExecutionRecord) {
        let pattern = record.query_pattern.clone();
        self.execution_history.push(record);
        self.total_queries += 1;

        // Recompute strategy stats for this pattern from full history.
        self.recompute_stats(&pattern);
    }

    /// Recommend the best strategy for a given query pattern.
    pub fn recommend_strategy(&self, query_pattern: &str) -> Option<StrategyStats> {
        self.strategy_cache.get(query_pattern).cloned()
    }

    /// Recommend the best attention mechanism for a given query pattern.
    pub fn recommend_attention(&self, query_pattern: &str) -> Option<String> {
        self.strategy_cache
            .get(query_pattern)
            .map(|s| s.best_attention.clone())
    }

    /// Percentage latency improvement of the most recent executions vs the
    /// earliest executions. Returns 0.0 if insufficient data.
    pub fn latency_improvement(&self) -> f64 {
        if self.execution_history.len() < 2 {
            return 0.0;
        }

        let n = self.execution_history.len();
        let window = (n / 2).max(1);

        let early_avg: f64 = self.execution_history[..window]
            .iter()
            .map(|r| r.latency_ms)
            .sum::<f64>()
            / window as f64;

        let recent_avg: f64 = self.execution_history[n - window..]
            .iter()
            .map(|r| r.latency_ms)
            .sum::<f64>()
            / window as f64;

        if early_avg == 0.0 {
            return 0.0;
        }

        ((early_avg - recent_avg) / early_avg) * 100.0
    }

    // -- internal helpers --

    fn recompute_stats(&mut self, pattern: &str) {
        let relevant: Vec<&ExecutionRecord> = self
            .execution_history
            .iter()
            .filter(|r| r.query_pattern == pattern)
            .collect();

        if relevant.is_empty() {
            return;
        }

        // Group by strategy.
        let mut by_strategy: HashMap<String, Vec<&ExecutionRecord>> = HashMap::new();
        for r in &relevant {
            by_strategy
                .entry(r.strategy_used.clone())
                .or_default()
                .push(r);
        }

        // Pick the strategy with the best combined score (high quality, low latency).
        let mut best: Option<StrategyStats> = None;
        let mut best_score = f64::NEG_INFINITY;

        for (strategy, records) in &by_strategy {
            let count = records.len();
            let avg_lat: f64 = records.iter().map(|r| r.latency_ms).sum::<f64>() / count as f64;
            let avg_q: f64 = records.iter().map(|r| r.quality_score).sum::<f64>() / count as f64;

            // Score: higher quality is better, lower latency is better.
            let score = avg_q - (avg_lat / 1000.0);

            // Best attention = the one used in the highest quality record.
            let best_att = records
                .iter()
                .max_by(|a, b| {
                    a.quality_score
                        .partial_cmp(&b.quality_score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|r| r.attention_mechanism.clone())
                .unwrap_or_default();

            if score > best_score {
                best_score = score;
                best = Some(StrategyStats {
                    strategy: strategy.clone(),
                    avg_latency_ms: avg_lat,
                    avg_quality: avg_q,
                    sample_count: count,
                    best_attention: best_att,
                });
            }
        }

        if let Some(stats) = best {
            self.strategy_cache.insert(pattern.to_string(), stats);
        }
    }
}

// ---------------------------------------------------------------------------
// Strategy Tracker
// ---------------------------------------------------------------------------

/// A record of a single strategy execution outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRecord {
    pub strategy: String,
    pub query_type: String,
    pub latency_ms: u64,
    pub success: bool,
    pub reward: f64,
    pub timestamp: DateTime<Utc>,
}

/// Tracks strategy success rates and selects optimal strategies per query type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyTracker {
    /// Full history of strategy records.
    pub records: Vec<StrategyRecord>,
    /// Per-strategy success tracking: maps strategy name to (successes, total).
    pub success_rates: HashMap<String, (u64, u64)>,
}

/// Summary statistics for the strategy tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyTrackerStats {
    pub total_queries: u64,
    pub strategies_tracked: usize,
    pub best_overall: Option<String>,
    pub avg_latency_ms: f64,
}

impl StrategyTracker {
    /// Create a new, empty strategy tracker.
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            success_rates: HashMap::new(),
        }
    }

    /// Record a strategy execution outcome.
    pub fn record(
        &mut self,
        strategy: &str,
        query_type: &str,
        latency_ms: u64,
        success: bool,
        reward: f64,
    ) {
        self.records.push(StrategyRecord {
            strategy: strategy.to_string(),
            query_type: query_type.to_string(),
            latency_ms,
            success,
            reward,
            timestamp: Utc::now(),
        });

        let entry = self
            .success_rates
            .entry(strategy.to_string())
            .or_insert((0, 0));
        if success {
            entry.0 += 1;
        }
        entry.1 += 1;
    }

    /// Return the success rate for a given strategy (0.0 if unknown).
    pub fn success_rate(&self, strategy: &str) -> f64 {
        match self.success_rates.get(strategy) {
            Some(&(successes, total)) if total > 0 => successes as f64 / total as f64,
            _ => 0.0,
        }
    }

    /// Return the strategy with the highest success rate for a given query type.
    /// Only considers strategies that have been used for this query type.
    pub fn best_strategy(&self, query_type: &str) -> Option<String> {
        // Group records by strategy for this query type.
        let mut by_strategy: HashMap<&str, (u64, u64)> = HashMap::new();
        for r in &self.records {
            if r.query_type == query_type {
                let entry = by_strategy.entry(&r.strategy).or_insert((0, 0));
                if r.success {
                    entry.0 += 1;
                }
                entry.1 += 1;
            }
        }

        by_strategy
            .into_iter()
            .filter(|(_, (_, total))| *total > 0)
            .max_by(|(_, (s1, t1)), (_, (s2, t2))| {
                let rate1 = *s1 as f64 / *t1 as f64;
                let rate2 = *s2 as f64 / *t2 as f64;
                rate1
                    .partial_cmp(&rate2)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(name, _)| name.to_string())
    }

    /// Return summary statistics.
    pub fn stats(&self) -> StrategyTrackerStats {
        let total_queries = self.records.len() as u64;
        let strategies_tracked = self.success_rates.len();
        let avg_latency_ms = if self.records.is_empty() {
            0.0
        } else {
            self.records
                .iter()
                .map(|r| r.latency_ms as f64)
                .sum::<f64>()
                / self.records.len() as f64
        };
        let best_overall = self
            .success_rates
            .iter()
            .filter(|(_, (_, total))| *total > 0)
            .max_by(|(_, (s1, t1)), (_, (s2, t2))| {
                let rate1 = *s1 as f64 / *t1 as f64;
                let rate2 = *s2 as f64 / *t2 as f64;
                rate1
                    .partial_cmp(&rate2)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(name, _)| name.clone());

        StrategyTrackerStats {
            total_queries,
            strategies_tracked,
            best_overall,
            avg_latency_ms,
        }
    }
}

impl Default for StrategyTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(
        pattern: &str,
        strategy: &str,
        attention: &str,
        latency: f64,
        quality: f64,
    ) -> ExecutionRecord {
        ExecutionRecord {
            query_hash: format!("hash_{}", pattern),
            query_pattern: pattern.to_string(),
            strategy_used: strategy.to_string(),
            attention_mechanism: attention.to_string(),
            latency_ms: latency,
            quality_score: quality,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_record_and_recommend() {
        let mut opt = DagOptimizer::new();
        opt.record(make_record("select", "parallel", "Flash", 10.0, 0.9));
        opt.record(make_record("select", "parallel", "Flash", 8.0, 0.95));

        let rec = opt.recommend_strategy("select");
        assert!(rec.is_some());
        let stats = rec.unwrap();
        assert_eq!(stats.strategy, "parallel");
        assert_eq!(stats.sample_count, 2);
        assert!((stats.avg_quality - 0.925).abs() < 0.001);
    }

    #[test]
    fn test_recommend_attention() {
        let mut opt = DagOptimizer::new();
        opt.record(make_record("join", "sequential", "Topological", 20.0, 0.8));
        opt.record(make_record("join", "sequential", "Flash", 15.0, 0.9));

        let att = opt.recommend_attention("join");
        assert!(att.is_some());
        // Flash had higher quality, so it should be best_attention.
        assert_eq!(att.unwrap(), "Flash");
    }

    #[test]
    fn test_latency_improvement() {
        let mut opt = DagOptimizer::new();
        // Early: high latency.
        opt.record(make_record("q", "s", "a", 100.0, 0.5));
        opt.record(make_record("q", "s", "a", 90.0, 0.5));
        // Recent: low latency.
        opt.record(make_record("q", "s", "a", 50.0, 0.8));
        opt.record(make_record("q", "s", "a", 40.0, 0.9));

        let improvement = opt.latency_improvement();
        // Early avg = 95, recent avg = 45, improvement ~52.6%
        assert!(
            improvement > 50.0,
            "Expected >50% improvement, got {}",
            improvement
        );
    }

    // -- StrategyTracker tests --

    #[test]
    fn test_record_strategy() {
        let mut tracker = StrategyTracker::new();
        tracker.record("parallel", "select", 10, true, 0.9);
        tracker.record("parallel", "select", 12, false, 0.3);

        assert_eq!(tracker.records.len(), 2);
        assert_eq!(tracker.success_rates.get("parallel"), Some(&(1, 2)));
    }

    #[test]
    fn test_success_rate() {
        let mut tracker = StrategyTracker::new();
        tracker.record("parallel", "select", 10, true, 0.9);
        tracker.record("parallel", "select", 12, true, 0.8);
        tracker.record("parallel", "select", 15, false, 0.2);

        let rate = tracker.success_rate("parallel");
        assert!((rate - 2.0 / 3.0).abs() < 0.001);

        // Unknown strategy returns 0.0.
        assert!((tracker.success_rate("unknown") - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_best_strategy() {
        let mut tracker = StrategyTracker::new();
        // parallel: 2/3 success for "select"
        tracker.record("parallel", "select", 10, true, 0.9);
        tracker.record("parallel", "select", 12, true, 0.8);
        tracker.record("parallel", "select", 15, false, 0.2);
        // sequential: 1/1 success for "select"
        tracker.record("sequential", "select", 20, true, 0.95);

        let best = tracker.best_strategy("select");
        assert_eq!(best, Some("sequential".to_string()));

        // Unknown query type returns None.
        assert!(tracker.best_strategy("unknown_type").is_none());
    }
}
