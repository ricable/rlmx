/**
 * Tests for the ReviewPipeline.
 * Mirrors rlmx-marketplace/src/review.rs tests.
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { ReviewPipeline } from '../src/review.js';
import { ReviewDecision, ReviewStatus, createPermission } from '../src/types.js';
import { MarketplaceError, MarketplaceErrorCode } from '../src/errors.js';

describe('ReviewPipeline', () => {
  let pipeline: ReviewPipeline;

  beforeEach(() => {
    pipeline = new ReviewPipeline();
  });

  describe('automated review', () => {
    it('should auto-pass agents with non-sensitive permissions', () => {
      const agentId = crypto.randomUUID();
      const subId = pipeline.submit(agentId);

      const result = pipeline.runAutomatedReview(subId, [
        createPermission('vec_search'),
      ]);

      expect(result.status).toBe(ReviewStatus.AutoPassed);
      expect(result.checks).toHaveLength(5);
      expect(result.checks.every((c) => c.passed)).toBe(true);
    });

    it('should auto-pass agents with no permissions', () => {
      const agentId = crypto.randomUUID();
      const subId = pipeline.submit(agentId);

      const result = pipeline.runAutomatedReview(subId, []);
      expect(result.status).toBe(ReviewStatus.AutoPassed);
    });

    it('should flag health_data for human review', () => {
      const agentId = crypto.randomUUID();
      const subId = pipeline.submit(agentId);

      const result = pipeline.runAutomatedReview(subId, [
        createPermission('health_data'),
      ]);

      expect(result.status).toBe(ReviewStatus.FlaggedForHuman);
    });

    it('should flag finance_data for human review', () => {
      const agentId = crypto.randomUUID();
      const subId = pipeline.submit(agentId);

      const result = pipeline.runAutomatedReview(subId, [
        createPermission('finance_data'),
      ]);

      expect(result.status).toBe(ReviewStatus.FlaggedForHuman);
    });

    it('should flag legal_data for human review', () => {
      const agentId = crypto.randomUUID();
      const subId = pipeline.submit(agentId);

      const result = pipeline.runAutomatedReview(subId, [
        createPermission('legal_data'),
      ]);

      expect(result.status).toBe(ReviewStatus.FlaggedForHuman);
    });

    it('should flag mixed permissions with one sensitive', () => {
      const agentId = crypto.randomUUID();
      const subId = pipeline.submit(agentId);

      const result = pipeline.runAutomatedReview(subId, [
        createPermission('vec_search'),
        createPermission('health_data'),
        createPermission('graph_query'),
      ]);

      expect(result.status).toBe(ReviewStatus.FlaggedForHuman);
    });

    it('should set completed_at on auto-pass', () => {
      const agentId = crypto.randomUUID();
      const subId = pipeline.submit(agentId);

      pipeline.runAutomatedReview(subId, []);

      const submission = pipeline.get(subId);
      expect(submission).toBeDefined();
      expect(submission!.completedAt).not.toBeNull();
    });

    it('should not set completed_at when flagged for human', () => {
      const agentId = crypto.randomUUID();
      const subId = pipeline.submit(agentId);

      pipeline.runAutomatedReview(subId, [createPermission('health_data')]);

      const submission = pipeline.get(subId);
      expect(submission!.completedAt).toBeNull();
    });
  });

  describe('human review', () => {
    it('should approve on human approval', () => {
      const agentId = crypto.randomUUID();
      const subId = pipeline.submit(agentId);

      pipeline.runAutomatedReview(subId, [createPermission('finance_data')]);

      const status = pipeline.completeHumanReview(
        subId,
        'reviewer_1',
        ReviewDecision.Approve,
        'Looks good',
      );

      expect(status).toBe(ReviewStatus.Approved);

      const submission = pipeline.get(subId)!;
      expect(submission.humanReview).not.toBeNull();
      expect(submission.humanReview!.reviewerId).toBe('reviewer_1');
      expect(submission.humanReview!.decision).toBe(ReviewDecision.Approve);
      expect(submission.completedAt).not.toBeNull();
    });

    it('should reject on human rejection', () => {
      const agentId = crypto.randomUUID();
      const subId = pipeline.submit(agentId);

      pipeline.runAutomatedReview(subId, [createPermission('legal_data')]);

      const status = pipeline.completeHumanReview(
        subId,
        'reviewer_2',
        ReviewDecision.Reject,
        'Insufficient privacy controls',
      );

      expect(status).toBe(ReviewStatus.Rejected);
    });

    it('should throw when reviewing non-flagged submission', () => {
      const agentId = crypto.randomUUID();
      const subId = pipeline.submit(agentId);

      // Auto-passed, not flagged
      pipeline.runAutomatedReview(subId, []);

      expect(() =>
        pipeline.completeHumanReview(
          subId,
          'reviewer',
          ReviewDecision.Approve,
          'n/a',
        ),
      ).toThrow(MarketplaceError);
    });

    it('should throw when reviewing non-existent submission', () => {
      expect(() =>
        pipeline.completeHumanReview(
          crypto.randomUUID(),
          'reviewer',
          ReviewDecision.Approve,
          'n/a',
        ),
      ).toThrow(MarketplaceError);
    });
  });

  describe('submission queries', () => {
    it('should return undefined for non-existent submission', () => {
      expect(pipeline.get(crypto.randomUUID())).toBeUndefined();
    });

    it('should list submissions by agent', () => {
      const agentId = crypto.randomUUID();
      pipeline.submit(agentId);
      pipeline.submit(agentId);
      pipeline.submit(crypto.randomUUID());

      const results = pipeline.byAgent(agentId);
      expect(results).toHaveLength(2);
    });

    it('should count pending submissions', () => {
      const a1 = crypto.randomUUID();
      const a2 = crypto.randomUUID();
      pipeline.submit(a1);
      pipeline.submit(a2);

      expect(pipeline.pendingCount()).toBe(2);

      // Complete one
      const subId = pipeline.byAgent(a1)[0].submissionId;
      pipeline.runAutomatedReview(subId, []);

      expect(pipeline.pendingCount()).toBe(1);
    });
  });

  describe('error handling', () => {
    it('should throw on automated review for non-existent submission', () => {
      expect(() =>
        pipeline.runAutomatedReview(crypto.randomUUID(), []),
      ).toThrow(MarketplaceError);
    });
  });
});
