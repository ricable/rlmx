/**
 * Marketplace error types.
 * Mirrors rlmx-marketplace/src/error.rs.
 */

import type { PublisherId, ReviewStatus } from './types.js';

/** Error codes for marketplace operations. */
export enum MarketplaceErrorCode {
  ListingNotFound = 'LISTING_NOT_FOUND',
  PublisherNotFound = 'PUBLISHER_NOT_FOUND',
  SubmissionNotFound = 'SUBMISSION_NOT_FOUND',
  DuplicateEmail = 'DUPLICATE_EMAIL',
  InvalidReviewState = 'INVALID_REVIEW_STATE',
  NotApproved = 'NOT_APPROVED',
  InvalidRevenueSplit = 'INVALID_REVENUE_SPLIT',
  PublisherNotVerified = 'PUBLISHER_NOT_VERIFIED',
  Internal = 'INTERNAL',
}

/** Base error class for marketplace operations. */
export class MarketplaceError extends Error {
  readonly code: MarketplaceErrorCode;
  readonly details?: unknown;

  constructor(code: MarketplaceErrorCode, message: string, details?: unknown) {
    super(message);
    this.name = 'MarketplaceError';
    this.code = code;
    this.details = details;
    Object.setPrototypeOf(this, new.target.prototype);
  }
}

// ---------------------------------------------------------------------------
// Factory helpers
// ---------------------------------------------------------------------------

export function listingNotFound(id: string): MarketplaceError {
  return new MarketplaceError(
    MarketplaceErrorCode.ListingNotFound,
    `Listing not found: ${id}`,
    { id },
  );
}

export function publisherNotFound(id: PublisherId): MarketplaceError {
  return new MarketplaceError(
    MarketplaceErrorCode.PublisherNotFound,
    `Publisher not found: ${id}`,
    { id },
  );
}

export function submissionNotFound(id: string): MarketplaceError {
  return new MarketplaceError(
    MarketplaceErrorCode.SubmissionNotFound,
    `Submission not found: ${id}`,
    { id },
  );
}

export function duplicateEmail(email: string): MarketplaceError {
  return new MarketplaceError(
    MarketplaceErrorCode.DuplicateEmail,
    `Duplicate email: ${email}`,
    { email },
  );
}

export function invalidReviewState(
  submissionId: string,
  status: ReviewStatus,
): MarketplaceError {
  return new MarketplaceError(
    MarketplaceErrorCode.InvalidReviewState,
    `Invalid review state for submission ${submissionId}: ${status}`,
    { submissionId, status },
  );
}

export function notApproved(id: string): MarketplaceError {
  return new MarketplaceError(
    MarketplaceErrorCode.NotApproved,
    `Agent not approved for publishing: ${id}`,
    { id },
  );
}

export function invalidRevenueSplit(): MarketplaceError {
  return new MarketplaceError(
    MarketplaceErrorCode.InvalidRevenueSplit,
    'Invalid revenue split: percentages must sum to 100',
  );
}

export function publisherNotVerified(id: PublisherId): MarketplaceError {
  return new MarketplaceError(
    MarketplaceErrorCode.PublisherNotVerified,
    `Publisher not verified: ${id}`,
    { id },
  );
}

export function internalError(message: string): MarketplaceError {
  return new MarketplaceError(
    MarketplaceErrorCode.Internal,
    `Internal error: ${message}`,
  );
}
