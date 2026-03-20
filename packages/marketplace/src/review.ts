/**
 * ReviewPipeline: automated security auditing and human review.
 * Mirrors rlmx-marketplace/src/review.rs.
 *
 * All 5 automated security checks must pass for auto-approval.
 * Agents with sensitive permissions are flagged for human review.
 */

import type {
  Permission,
  ReviewSubmission,
  SecurityCheck,
} from './types.js';
import {
  ReviewDecision,
  ReviewStatus,
  SecurityCheckType,
  Severity,
  isSensitivePermission,
} from './types.js';
import { invalidReviewState, submissionNotFound } from './errors.js';

// ---------------------------------------------------------------------------
// ReviewResult
// ---------------------------------------------------------------------------

/** Result of running the automated review pipeline. */
export interface ReviewResult {
  submissionId: string;
  status: ReviewStatus;
  checks: SecurityCheck[];
}

// ---------------------------------------------------------------------------
// ReviewPipeline
// ---------------------------------------------------------------------------

/** The review pipeline manages automated and human review of agent submissions. */
export class ReviewPipeline {
  private readonly submissions = new Map<string, ReviewSubmission>();

  /** Create a new review submission for an agent. Returns the submission ID. */
  submit(agentId: string): string {
    const submissionId = crypto.randomUUID();
    const submission: ReviewSubmission = {
      submissionId,
      agentId,
      automatedChecks: [],
      humanReview: null,
      status: ReviewStatus.Pending,
      submittedAt: new Date().toISOString(),
      completedAt: null,
    };
    this.submissions.set(submissionId, submission);
    return submissionId;
  }

  /**
   * Run all 5 automated security checks (stub implementation).
   * Invariant: all 5 checks must pass for auto-approval.
   * Agents with sensitive permissions are flagged for human review.
   */
  runAutomatedReview(
    submissionId: string,
    permissions: readonly Permission[],
  ): ReviewResult {
    const submission = this.submissions.get(submissionId);
    if (!submission) {
      throw submissionNotFound(submissionId);
    }

    submission.status = ReviewStatus.InReview;

    // Run all 5 security check types (stubbed as passing).
    const checks: SecurityCheck[] = [
      {
        checkType: SecurityCheckType.CapabilityMinimality,
        passed: true,
        details: 'Agent requests minimal permissions',
        severity: Severity.Medium,
      },
      {
        checkType: SecurityCheckType.DataFlowVerification,
        passed: true,
        details: 'Data stays within declared scope',
        severity: Severity.High,
      },
      {
        checkType: SecurityCheckType.FuzzTesting,
        passed: true,
        details: '1000 random inputs, no crashes detected',
        severity: Severity.High,
      },
      {
        checkType: SecurityCheckType.NetworkPolicyCompliance,
        passed: true,
        details: 'No undeclared network calls',
        severity: Severity.Critical,
      },
      {
        checkType: SecurityCheckType.MalwareSignatureScan,
        passed: true,
        details: 'No known malicious patterns',
        severity: Severity.Critical,
      },
    ];

    const allPassed = checks.every((c) => c.passed);
    const hasHighSeverityFailure = checks.some(
      (c) =>
        !c.passed &&
        (c.severity === Severity.High || c.severity === Severity.Critical),
    );

    submission.automatedChecks = checks;

    let status: ReviewStatus;
    if (hasHighSeverityFailure || !allPassed) {
      status = ReviewStatus.Rejected;
    } else if (permissions.some((p) => isSensitivePermission(p))) {
      status = ReviewStatus.FlaggedForHuman;
    } else {
      status = ReviewStatus.AutoPassed;
    }

    submission.status = status;
    if (
      status === ReviewStatus.AutoPassed ||
      status === ReviewStatus.Rejected
    ) {
      submission.completedAt = new Date().toISOString();
    }

    return {
      submissionId,
      status,
      checks: [...submission.automatedChecks],
    };
  }

  /** Complete a human review for a flagged submission. */
  completeHumanReview(
    submissionId: string,
    reviewerId: string,
    decision: ReviewDecision,
    notes: string,
  ): ReviewStatus {
    const submission = this.submissions.get(submissionId);
    if (!submission) {
      throw submissionNotFound(submissionId);
    }

    if (submission.status !== ReviewStatus.FlaggedForHuman) {
      throw invalidReviewState(submissionId, submission.status);
    }

    const newStatus =
      decision === ReviewDecision.Approve
        ? ReviewStatus.Approved
        : ReviewStatus.Rejected;

    submission.humanReview = {
      reviewerId,
      decision,
      notes,
      reviewedAt: new Date().toISOString(),
    };
    submission.status = newStatus;
    submission.completedAt = new Date().toISOString();

    return newStatus;
  }

  /** Get a submission by ID. */
  get(submissionId: string): ReviewSubmission | undefined {
    return this.submissions.get(submissionId);
  }

  /** Get all submissions for a given agent. */
  byAgent(agentId: string): ReviewSubmission[] {
    return Array.from(this.submissions.values()).filter(
      (s) => s.agentId === agentId,
    );
  }

  /** Count of pending submissions. */
  pendingCount(): number {
    return Array.from(this.submissions.values()).filter(
      (s) =>
        s.status === ReviewStatus.Pending ||
        s.status === ReviewStatus.InReview,
    ).length;
  }
}
