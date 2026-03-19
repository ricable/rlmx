//! Feedback engine for evolved function quality management.

use std::collections::HashMap;

use tracing::warn;

use crate::types::{FeedbackDecision, FunctionId, ScoreResult};

/// Tracks evaluation history and produces feedback decisions.
#[derive(Debug, Default)]
pub struct FeedbackEngine {
    eval_history: HashMap<FunctionId, Vec<ScoreResult>>,
}

impl FeedbackEngine {
    /// Create a new empty feedback engine.
    pub fn new() -> Self {
        Self {
            eval_history: HashMap::new(),
        }
    }

    /// Record a score result for a function.
    pub fn record_eval(&mut self, function_id: FunctionId, score_result: ScoreResult) {
        self.eval_history
            .entry(function_id)
            .or_default()
            .push(score_result);
    }

    /// Review the last 5 results for a function and produce a decision.
    ///
    /// - **Kill**: 3 or more of the last 5 have correctness < 0.5
    /// - **Improve**: average overall of last 5 < 0.5
    /// - **Keep**: otherwise
    pub fn review(&self, function_id: FunctionId) -> FeedbackDecision {
        let results = match self.eval_history.get(&function_id) {
            Some(r) if !r.is_empty() => r,
            _ => return FeedbackDecision::Keep,
        };

        let last_5: Vec<&ScoreResult> = results.iter().rev().take(5).collect();

        let low_correctness_count = last_5
            .iter()
            .filter(|s| s.correctness < 0.5)
            .count();

        if low_correctness_count >= 3 {
            warn!(
                function_id = %function_id,
                low_correctness_count,
                "killing function due to repeated low correctness"
            );
            return FeedbackDecision::Kill {
                reason: format!(
                    "{low_correctness_count} of last {} evaluations had correctness < 0.5",
                    last_5.len()
                ),
            };
        }

        let avg_overall: f64 =
            last_5.iter().map(|s| s.overall).sum::<f64>() / last_5.len() as f64;

        if avg_overall < 0.5 {
            return FeedbackDecision::Improve {
                reason: format!(
                    "average overall score {avg_overall:.3} is below 0.5 threshold"
                ),
            };
        }

        FeedbackDecision::Keep
    }

    /// Get the last N score results for a function.
    pub fn last_n_results(
        &self,
        function_id: FunctionId,
        n: usize,
    ) -> Vec<&ScoreResult> {
        match self.eval_history.get(&function_id) {
            Some(results) => results.iter().rev().take(n).collect(),
            None => Vec::new(),
        }
    }

    /// Produce a leaderboard sorted by average overall score (descending).
    pub fn leaderboard(&self) -> Vec<(FunctionId, f64)> {
        let mut board: Vec<(FunctionId, f64)> = self
            .eval_history
            .iter()
            .filter(|(_, results)| !results.is_empty())
            .map(|(id, results)| {
                let avg = results.iter().map(|s| s.overall).sum::<f64>()
                    / results.len() as f64;
                (*id, avg)
            })
            .collect();

        board.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        board
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_score(correctness: f64, overall: f64) -> ScoreResult {
        ScoreResult {
            overall,
            correctness,
            safety: 1.0,
            latency_score: 1.0,
            cost_score: 1.0,
        }
    }

    #[test]
    fn review_no_history_returns_keep() {
        let engine = FeedbackEngine::new();
        assert_eq!(engine.review(FunctionId::new()), FeedbackDecision::Keep);
    }

    #[test]
    fn review_all_good_returns_keep() {
        let mut engine = FeedbackEngine::new();
        let id = FunctionId::new();
        for _ in 0..5 {
            engine.record_eval(id, make_score(1.0, 0.9));
        }
        assert_eq!(engine.review(id), FeedbackDecision::Keep);
    }

    #[test]
    fn review_kill_three_low_correctness() {
        let mut engine = FeedbackEngine::new();
        let id = FunctionId::new();
        engine.record_eval(id, make_score(0.3, 0.3));
        engine.record_eval(id, make_score(0.8, 0.8));
        engine.record_eval(id, make_score(0.2, 0.2));
        engine.record_eval(id, make_score(0.1, 0.1));
        engine.record_eval(id, make_score(0.9, 0.9));
        match engine.review(id) {
            FeedbackDecision::Kill { reason } => {
                assert!(reason.contains("3"));
            }
            other => panic!("expected Kill, got {other:?}"),
        }
    }

    #[test]
    fn review_improve_low_avg_overall() {
        let mut engine = FeedbackEngine::new();
        let id = FunctionId::new();
        // All have correctness >= 0.5 (so no kill), but low overall
        for _ in 0..5 {
            engine.record_eval(id, make_score(0.5, 0.3));
        }
        match engine.review(id) {
            FeedbackDecision::Improve { reason } => {
                assert!(reason.contains("0.3"));
            }
            other => panic!("expected Improve, got {other:?}"),
        }
    }

    #[test]
    fn review_only_last_5_matter() {
        let mut engine = FeedbackEngine::new();
        let id = FunctionId::new();
        // 10 terrible old scores
        for _ in 0..10 {
            engine.record_eval(id, make_score(0.0, 0.0));
        }
        // 5 great new scores
        for _ in 0..5 {
            engine.record_eval(id, make_score(1.0, 0.9));
        }
        assert_eq!(engine.review(id), FeedbackDecision::Keep);
    }

    #[test]
    fn review_kill_takes_priority_over_improve() {
        let mut engine = FeedbackEngine::new();
        let id = FunctionId::new();
        // 4 with low correctness AND low overall
        for _ in 0..4 {
            engine.record_eval(id, make_score(0.1, 0.1));
        }
        engine.record_eval(id, make_score(0.8, 0.8));
        match engine.review(id) {
            FeedbackDecision::Kill { .. } => {}
            other => panic!("expected Kill, got {other:?}"),
        }
    }

    #[test]
    fn review_fewer_than_5_evals() {
        let mut engine = FeedbackEngine::new();
        let id = FunctionId::new();
        engine.record_eval(id, make_score(1.0, 0.8));
        assert_eq!(engine.review(id), FeedbackDecision::Keep);
    }

    #[test]
    fn review_exactly_at_threshold() {
        let mut engine = FeedbackEngine::new();
        let id = FunctionId::new();
        // correctness exactly 0.5 should NOT count as "< 0.5"
        for _ in 0..5 {
            engine.record_eval(id, make_score(0.5, 0.5));
        }
        assert_eq!(engine.review(id), FeedbackDecision::Keep);
    }

    #[test]
    fn last_n_results_empty() {
        let engine = FeedbackEngine::new();
        assert!(engine.last_n_results(FunctionId::new(), 5).is_empty());
    }

    #[test]
    fn last_n_results_returns_most_recent() {
        let mut engine = FeedbackEngine::new();
        let id = FunctionId::new();
        engine.record_eval(id, make_score(0.1, 0.1));
        engine.record_eval(id, make_score(0.5, 0.5));
        engine.record_eval(id, make_score(0.9, 0.9));
        let last2 = engine.last_n_results(id, 2);
        assert_eq!(last2.len(), 2);
        assert!((last2[0].correctness - 0.9).abs() < f64::EPSILON);
        assert!((last2[1].correctness - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn last_n_results_fewer_than_n() {
        let mut engine = FeedbackEngine::new();
        let id = FunctionId::new();
        engine.record_eval(id, make_score(0.7, 0.7));
        let results = engine.last_n_results(id, 10);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn leaderboard_empty() {
        let engine = FeedbackEngine::new();
        assert!(engine.leaderboard().is_empty());
    }

    #[test]
    fn leaderboard_sorted_descending() {
        let mut engine = FeedbackEngine::new();
        let low = FunctionId::new();
        let high = FunctionId::new();
        let mid = FunctionId::new();

        engine.record_eval(low, make_score(0.2, 0.2));
        engine.record_eval(high, make_score(1.0, 0.95));
        engine.record_eval(mid, make_score(0.5, 0.6));

        let board = engine.leaderboard();
        assert_eq!(board.len(), 3);
        assert_eq!(board[0].0, high);
        assert_eq!(board[1].0, mid);
        assert_eq!(board[2].0, low);
    }

    #[test]
    fn leaderboard_averages_multiple_evals() {
        let mut engine = FeedbackEngine::new();
        let id = FunctionId::new();
        engine.record_eval(id, make_score(1.0, 0.6));
        engine.record_eval(id, make_score(1.0, 0.8));
        let board = engine.leaderboard();
        assert_eq!(board.len(), 1);
        assert!((board[0].1 - 0.7).abs() < 1e-10);
    }

    #[test]
    fn review_two_low_correctness_not_kill() {
        let mut engine = FeedbackEngine::new();
        let id = FunctionId::new();
        engine.record_eval(id, make_score(0.1, 0.8));
        engine.record_eval(id, make_score(0.1, 0.8));
        engine.record_eval(id, make_score(0.9, 0.8));
        engine.record_eval(id, make_score(0.9, 0.8));
        engine.record_eval(id, make_score(0.9, 0.8));
        // Only 2 of last 5 have low correctness, should Keep
        assert_eq!(engine.review(id), FeedbackDecision::Keep);
    }
}
