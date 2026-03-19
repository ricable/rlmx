//! Scoring engine for evolved functions.

use std::collections::HashSet;

use crate::types::ScoreResult;

/// Stateless scoring engine that evaluates function execution quality.
#[derive(Debug, Default)]
pub struct ScoreEngine;

impl ScoreEngine {
    /// Create a new ScoreEngine.
    pub fn new() -> Self {
        Self
    }

    /// Deep-equality scorer. Returns 1.0 if values match exactly, 0.0 otherwise.
    pub fn exact_match(
        actual: &serde_json::Value,
        expected: &serde_json::Value,
    ) -> f64 {
        if actual == expected {
            1.0
        } else {
            0.0
        }
    }

    /// Jaccard word-set similarity between two strings.
    /// Returns the size of the intersection divided by the union of word sets.
    pub fn semantic_similarity(a: &str, b: &str) -> f64 {
        let words_a: HashSet<&str> = a.split_whitespace().collect();
        let words_b: HashSet<&str> = b.split_whitespace().collect();

        if words_a.is_empty() && words_b.is_empty() {
            return 1.0;
        }

        let intersection = words_a.intersection(&words_b).count() as f64;
        let union = words_a.union(&words_b).count() as f64;

        if union == 0.0 {
            return 0.0;
        }

        intersection / union
    }

    /// Compute the overall score using the weighted formula.
    ///
    /// Formula:
    /// ```text
    /// overall = correctness * 0.50
    ///         + safety * 0.25
    ///         + latency_score * 0.15
    ///         + cost_score * 0.10
    /// ```
    ///
    /// Where:
    /// - `latency_score = clamp(1.0 - actual_ms / timeout_ms, 0, 1)`
    /// - `cost_score = clamp(1.0 - cost / budget_limit, 0, 1)`
    pub fn compute_overall(
        correctness: f64,
        safety: f64,
        latency_ms: u64,
        timeout_ms: u64,
        cost_microcents: u64,
        budget_limit_microcents: u64,
    ) -> ScoreResult {
        let latency_score = if timeout_ms == 0 {
            0.0
        } else {
            (1.0 - latency_ms as f64 / timeout_ms as f64).clamp(0.0, 1.0)
        };

        let cost_score = if budget_limit_microcents == 0 {
            0.0
        } else {
            (1.0 - cost_microcents as f64 / budget_limit_microcents as f64).clamp(0.0, 1.0)
        };

        let overall = correctness * 0.50
            + safety * 0.25
            + latency_score * 0.15
            + cost_score * 0.10;

        ScoreResult {
            overall,
            correctness,
            safety,
            latency_score,
            cost_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // --- exact_match ---

    #[test]
    fn exact_match_equal_numbers() {
        assert_eq!(ScoreEngine::exact_match(&json!(42), &json!(42)), 1.0);
    }

    #[test]
    fn exact_match_unequal_numbers() {
        assert_eq!(ScoreEngine::exact_match(&json!(42), &json!(43)), 0.0);
    }

    #[test]
    fn exact_match_equal_strings() {
        assert_eq!(
            ScoreEngine::exact_match(&json!("hello"), &json!("hello")),
            1.0
        );
    }

    #[test]
    fn exact_match_unequal_types() {
        assert_eq!(ScoreEngine::exact_match(&json!(42), &json!("42")), 0.0);
    }

    #[test]
    fn exact_match_nested_objects() {
        let a = json!({"a": [1, 2], "b": "c"});
        let b = json!({"a": [1, 2], "b": "c"});
        assert_eq!(ScoreEngine::exact_match(&a, &b), 1.0);
    }

    #[test]
    fn exact_match_nested_mismatch() {
        let a = json!({"a": [1, 2]});
        let b = json!({"a": [1, 3]});
        assert_eq!(ScoreEngine::exact_match(&a, &b), 0.0);
    }

    #[test]
    fn exact_match_null() {
        assert_eq!(
            ScoreEngine::exact_match(&json!(null), &json!(null)),
            1.0
        );
    }

    // --- semantic_similarity ---

    #[test]
    fn semantic_identical() {
        assert!((ScoreEngine::semantic_similarity("hello world", "hello world") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn semantic_disjoint() {
        assert!((ScoreEngine::semantic_similarity("hello world", "foo bar")).abs() < f64::EPSILON);
    }

    #[test]
    fn semantic_partial_overlap() {
        let sim = ScoreEngine::semantic_similarity("the quick brown fox", "the slow brown dog");
        // intersection: {the, brown} = 2, union: {the, quick, brown, fox, slow, dog} = 6
        assert!((sim - 2.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn semantic_both_empty() {
        assert!((ScoreEngine::semantic_similarity("", "") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn semantic_one_empty() {
        assert!(ScoreEngine::semantic_similarity("hello", "").abs() < f64::EPSILON);
    }

    // --- compute_overall ---

    #[test]
    fn overall_perfect_score() {
        let r = ScoreEngine::compute_overall(1.0, 1.0, 0, 1000, 0, 10000);
        // 1.0*0.5 + 1.0*0.25 + 1.0*0.15 + 1.0*0.10 = 1.0
        assert!((r.overall - 1.0).abs() < 1e-10);
    }

    #[test]
    fn overall_zero_score() {
        let r = ScoreEngine::compute_overall(0.0, 0.0, 2000, 1000, 20000, 10000);
        // 0 + 0 + 0 (latency capped) + 0 (cost capped) = 0
        assert!((r.overall).abs() < 1e-10);
    }

    #[test]
    fn overall_mixed() {
        let r = ScoreEngine::compute_overall(0.8, 0.6, 500, 1000, 3000, 10000);
        // latency = 1 - 0.5 = 0.5
        // cost = 1 - 0.3 = 0.7
        // overall = 0.8*0.5 + 0.6*0.25 + 0.5*0.15 + 0.7*0.10
        //         = 0.4 + 0.15 + 0.075 + 0.07 = 0.695
        assert!((r.overall - 0.695).abs() < 1e-10);
        assert!((r.latency_score - 0.5).abs() < 1e-10);
        assert!((r.cost_score - 0.7).abs() < 1e-10);
    }

    #[test]
    fn overall_latency_exceeds_timeout_clamped() {
        let r = ScoreEngine::compute_overall(1.0, 1.0, 5000, 1000, 0, 10000);
        assert!((r.latency_score).abs() < 1e-10);
    }

    #[test]
    fn overall_cost_exceeds_budget_clamped() {
        let r = ScoreEngine::compute_overall(1.0, 1.0, 0, 1000, 50000, 10000);
        assert!((r.cost_score).abs() < 1e-10);
    }

    #[test]
    fn overall_zero_timeout_yields_zero_latency_score() {
        let r = ScoreEngine::compute_overall(1.0, 1.0, 100, 0, 0, 10000);
        assert!((r.latency_score).abs() < 1e-10);
    }

    #[test]
    fn overall_zero_budget_yields_zero_cost_score() {
        let r = ScoreEngine::compute_overall(1.0, 1.0, 0, 1000, 100, 0);
        assert!((r.cost_score).abs() < 1e-10);
    }

    #[test]
    fn overall_stores_components() {
        let r = ScoreEngine::compute_overall(0.9, 0.8, 200, 1000, 1000, 5000);
        assert!((r.correctness - 0.9).abs() < 1e-10);
        assert!((r.safety - 0.8).abs() < 1e-10);
    }
}
