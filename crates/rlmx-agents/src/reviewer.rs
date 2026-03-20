use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::AgentId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewVerdict {
    Approved,
    NeedsChanges,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Review {
    pub id: Uuid,
    pub target: String,
    pub verdict: ReviewVerdict,
    pub comments: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// Reviewer agent — evaluates targets and produces review verdicts.
pub struct ReviewerAgent {
    pub id: AgentId,
    pub reviews: Vec<Review>,
}

impl ReviewerAgent {
    pub fn new() -> Self {
        Self {
            id: AgentId::new(),
            reviews: Vec::new(),
        }
    }

    /// Review a target with the given data and produce a verdict.
    pub fn review(&mut self, target: &str, data: &serde_json::Value) -> Review {
        let mut comments = Vec::new();

        // Heuristic review logic based on data structure
        let has_errors = data.get("errors").is_some();
        let has_warnings = data.get("warnings").is_some();
        let quality_score = data
            .get("quality_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);

        let verdict = if has_errors {
            comments.push("Errors detected in submission".into());
            ReviewVerdict::Rejected
        } else if has_warnings || quality_score < 0.7 {
            comments.push(format!(
                "Quality score {:.2} below threshold (0.70)",
                quality_score
            ));
            if has_warnings {
                comments.push("Warnings present — please address".into());
            }
            ReviewVerdict::NeedsChanges
        } else {
            comments.push(format!(
                "Quality score {:.2} meets standards",
                quality_score
            ));
            ReviewVerdict::Approved
        };

        let review = Review {
            id: Uuid::new_v4(),
            target: target.to_string(),
            verdict,
            comments,
            timestamp: Utc::now(),
        };

        self.reviews.push(review.clone());
        review
    }

    /// Return number of reviews performed.
    pub fn review_count(&self) -> usize {
        self.reviews.len()
    }
}

impl Default for ReviewerAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_approved() {
        let mut reviewer = ReviewerAgent::new();
        let data = serde_json::json!({"quality_score": 0.9});
        let review = reviewer.review("module-a", &data);
        assert_eq!(review.verdict, ReviewVerdict::Approved);
        assert_eq!(review.target, "module-a");
        assert!(!review.comments.is_empty());
    }

    #[test]
    fn test_review_needs_changes_low_quality() {
        let mut reviewer = ReviewerAgent::new();
        let data = serde_json::json!({"quality_score": 0.5});
        let review = reviewer.review("module-b", &data);
        assert_eq!(review.verdict, ReviewVerdict::NeedsChanges);
    }

    #[test]
    fn test_review_needs_changes_warnings() {
        let mut reviewer = ReviewerAgent::new();
        let data = serde_json::json!({"warnings": ["warn1"], "quality_score": 0.8});
        let review = reviewer.review("module-c", &data);
        assert_eq!(review.verdict, ReviewVerdict::NeedsChanges);
    }

    #[test]
    fn test_review_rejected_errors() {
        let mut reviewer = ReviewerAgent::new();
        let data = serde_json::json!({"errors": ["err1"], "quality_score": 0.95});
        let review = reviewer.review("module-d", &data);
        assert_eq!(review.verdict, ReviewVerdict::Rejected);
    }

    #[test]
    fn test_review_count() {
        let mut reviewer = ReviewerAgent::new();
        let data = serde_json::json!({});
        reviewer.review("a", &data);
        reviewer.review("b", &data);
        assert_eq!(reviewer.review_count(), 2);
    }

    #[test]
    fn test_default_quality_score() {
        let mut reviewer = ReviewerAgent::new();
        let data = serde_json::json!({});
        let review = reviewer.review("no-score", &data);
        // Default 0.5 < 0.7 => NeedsChanges
        assert_eq!(review.verdict, ReviewVerdict::NeedsChanges);
    }
}
