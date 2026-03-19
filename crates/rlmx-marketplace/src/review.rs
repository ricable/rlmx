//! Review pipeline: automated security auditing and human review.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{Permission, ReviewStatus, SecurityCheck, SecurityCheckType, Severity};
use crate::error::MarketplaceError;

/// A review submission tracking entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewSubmission {
    pub submission_id: Uuid,
    pub agent_id: Uuid,
    pub automated_checks: Vec<SecurityCheck>,
    pub human_review: Option<HumanReview>,
    pub status: ReviewStatus,
    pub submitted_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Human review decision and notes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanReview {
    pub reviewer_id: String,
    pub decision: ReviewDecision,
    pub notes: String,
    pub reviewed_at: DateTime<Utc>,
}

/// Decision made by a human reviewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewDecision {
    Approve,
    Reject,
}

/// Result of running the automated review pipeline.
#[derive(Debug, Clone)]
pub struct ReviewResult {
    pub submission_id: Uuid,
    pub status: ReviewStatus,
    pub checks: Vec<SecurityCheck>,
}

/// The review pipeline manages automated and human review of agent submissions.
#[derive(Debug, Default)]
pub struct ReviewPipeline {
    submissions: HashMap<Uuid, ReviewSubmission>,
}

impl ReviewPipeline {
    pub fn new() -> Self {
        Self {
            submissions: HashMap::new(),
        }
    }

    /// Create a new review submission for an agent.
    pub fn submit(&mut self, agent_id: Uuid) -> Uuid {
        let submission_id = Uuid::new_v4();
        let submission = ReviewSubmission {
            submission_id,
            agent_id,
            automated_checks: Vec::new(),
            human_review: None,
            status: ReviewStatus::Pending,
            submitted_at: Utc::now(),
            completed_at: None,
        };
        tracing::info!(submission_id = %submission_id, agent_id = %agent_id, "review submitted");
        self.submissions.insert(submission_id, submission);
        submission_id
    }

    /// Run all 5 automated security checks (stub implementation).
    /// Invariant: all 5 checks must pass for auto-approval.
    /// Agents with sensitive permissions are flagged for human review.
    pub fn run_automated_review(
        &mut self,
        submission_id: &Uuid,
        permissions: &[Permission],
    ) -> Result<ReviewResult, MarketplaceError> {
        let submission = self
            .submissions
            .get_mut(submission_id)
            .ok_or(MarketplaceError::SubmissionNotFound(*submission_id))?;

        submission.status = ReviewStatus::InReview;

        // Run all 5 security check types (stubbed as passing).
        let checks = vec![
            SecurityCheck {
                check_type: SecurityCheckType::CapabilityMinimality,
                passed: true,
                details: "Agent requests minimal permissions".into(),
                severity: Severity::Medium,
            },
            SecurityCheck {
                check_type: SecurityCheckType::DataFlowVerification,
                passed: true,
                details: "Data stays within declared scope".into(),
                severity: Severity::High,
            },
            SecurityCheck {
                check_type: SecurityCheckType::FuzzTesting,
                passed: true,
                details: "1000 random inputs, no crashes detected".into(),
                severity: Severity::High,
            },
            SecurityCheck {
                check_type: SecurityCheckType::NetworkPolicyCompliance,
                passed: true,
                details: "No undeclared network calls".into(),
                severity: Severity::Critical,
            },
            SecurityCheck {
                check_type: SecurityCheckType::MalwareSignatureScan,
                passed: true,
                details: "No known malicious patterns".into(),
                severity: Severity::Critical,
            },
        ];

        let all_passed = checks.iter().all(|c| c.passed);
        let has_high_severity_failure = checks
            .iter()
            .any(|c| !c.passed && c.severity >= Severity::High);

        submission.automated_checks = checks.clone();

        let status = if has_high_severity_failure || !all_passed {
            ReviewStatus::Rejected
        } else if permissions.iter().any(|p| p.is_sensitive()) {
            ReviewStatus::FlaggedForHuman
        } else {
            ReviewStatus::AutoPassed
        };

        submission.status = status;
        if matches!(status, ReviewStatus::AutoPassed | ReviewStatus::Rejected) {
            submission.completed_at = Some(Utc::now());
        }

        tracing::info!(
            submission_id = %submission_id,
            status = ?status,
            "automated review completed"
        );

        Ok(ReviewResult {
            submission_id: *submission_id,
            status,
            checks: submission.automated_checks.clone(),
        })
    }

    /// Complete a human review for a flagged submission.
    pub fn complete_human_review(
        &mut self,
        submission_id: &Uuid,
        reviewer_id: String,
        decision: ReviewDecision,
        notes: String,
    ) -> Result<ReviewStatus, MarketplaceError> {
        let submission = self
            .submissions
            .get_mut(submission_id)
            .ok_or(MarketplaceError::SubmissionNotFound(*submission_id))?;

        if submission.status != ReviewStatus::FlaggedForHuman {
            return Err(MarketplaceError::InvalidReviewState(
                *submission_id,
                submission.status,
            ));
        }

        let human_review = HumanReview {
            reviewer_id,
            decision,
            notes,
            reviewed_at: Utc::now(),
        };

        let new_status = match decision {
            ReviewDecision::Approve => ReviewStatus::Approved,
            ReviewDecision::Reject => ReviewStatus::Rejected,
        };

        submission.human_review = Some(human_review);
        submission.status = new_status;
        submission.completed_at = Some(Utc::now());

        tracing::info!(
            submission_id = %submission_id,
            status = ?new_status,
            "human review completed"
        );

        Ok(new_status)
    }

    /// Get a submission by ID.
    pub fn get(&self, submission_id: &Uuid) -> Option<&ReviewSubmission> {
        self.submissions.get(submission_id)
    }

    /// Get all submissions for a given agent.
    pub fn by_agent(&self, agent_id: &Uuid) -> Vec<&ReviewSubmission> {
        self.submissions
            .values()
            .filter(|s| s.agent_id == *agent_id)
            .collect()
    }

    /// Count of pending submissions.
    pub fn pending_count(&self) -> usize {
        self.submissions
            .values()
            .filter(|s| matches!(s.status, ReviewStatus::Pending | ReviewStatus::InReview))
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automated_review_auto_passes() {
        let mut pipeline = ReviewPipeline::new();
        let agent_id = Uuid::new_v4();
        let sub_id = pipeline.submit(agent_id);

        let result = pipeline
            .run_automated_review(&sub_id, &[Permission::new("vec_search")])
            .unwrap();

        assert_eq!(result.status, ReviewStatus::AutoPassed);
        assert_eq!(result.checks.len(), 5);
    }

    #[test]
    fn sensitive_permissions_flagged_for_human() {
        let mut pipeline = ReviewPipeline::new();
        let agent_id = Uuid::new_v4();
        let sub_id = pipeline.submit(agent_id);

        let result = pipeline
            .run_automated_review(&sub_id, &[Permission::new("health_data")])
            .unwrap();

        assert_eq!(result.status, ReviewStatus::FlaggedForHuman);
    }

    #[test]
    fn human_review_approve() {
        let mut pipeline = ReviewPipeline::new();
        let agent_id = Uuid::new_v4();
        let sub_id = pipeline.submit(agent_id);

        pipeline
            .run_automated_review(&sub_id, &[Permission::new("finance_data")])
            .unwrap();

        let status = pipeline
            .complete_human_review(
                &sub_id,
                "reviewer_1".into(),
                ReviewDecision::Approve,
                "Looks good".into(),
            )
            .unwrap();

        assert_eq!(status, ReviewStatus::Approved);
    }

    #[test]
    fn human_review_reject() {
        let mut pipeline = ReviewPipeline::new();
        let agent_id = Uuid::new_v4();
        let sub_id = pipeline.submit(agent_id);

        pipeline
            .run_automated_review(&sub_id, &[Permission::new("legal_data")])
            .unwrap();

        let status = pipeline
            .complete_human_review(
                &sub_id,
                "reviewer_2".into(),
                ReviewDecision::Reject,
                "Insufficient privacy controls".into(),
            )
            .unwrap();

        assert_eq!(status, ReviewStatus::Rejected);
    }

    #[test]
    fn cannot_human_review_non_flagged() {
        let mut pipeline = ReviewPipeline::new();
        let agent_id = Uuid::new_v4();
        let sub_id = pipeline.submit(agent_id);

        // Auto-passed, not flagged
        pipeline.run_automated_review(&sub_id, &[]).unwrap();

        let result = pipeline.complete_human_review(
            &sub_id,
            "reviewer".into(),
            ReviewDecision::Approve,
            "n/a".into(),
        );

        assert!(result.is_err());
    }
}
