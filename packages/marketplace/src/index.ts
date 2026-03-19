/**
 * @aix/marketplace — Agent marketplace for the @aix ecosystem.
 *
 * Ports the rlmx-marketplace Rust crate (DDD-010, ADR-014) to TypeScript.
 * Manages publishing, discovery, installation, billing, review, and analytics
 * for the RLMX agent ecosystem.
 */

// Aggregate root
export { Marketplace } from './marketplace.js';

// Registry
export {
  AgentRegistry,
  createDraftListing,
  addRating,
  recordInstall,
} from './registry.js';

// Billing
export { BillingEngine } from './billing.js';

// Review
export { ReviewPipeline } from './review.js';
export type { ReviewResult } from './review.js';

// Publisher
export {
  PublisherPortal,
  addAgentToPublisher,
  removeAgentFromPublisher,
  updateReputation,
} from './publisher.js';

// Featured
export { FeaturedEngine } from './featured.js';

// Analytics
export { MarketplaceAnalytics, retentionRate } from './analytics.js';

// Events
export type {
  MarketplaceEvent,
  MarketplaceEventType,
  MarketplaceEventOf,
  AgentSubmittedEvent,
  ReviewStartedEvent,
  ReviewCompletedEvent,
  AgentPublishedEvent,
  AgentInstalledEvent,
  AgentUninstalledEvent,
  AgentRatedEvent,
  AgentSuspendedEvent,
  PayoutProcessedEvent,
  PackCreatedEvent,
} from './events.js';
export { MARKETPLACE_EVENT_TYPES } from './events.js';

// Errors
export {
  MarketplaceError,
  MarketplaceErrorCode,
  listingNotFound,
  publisherNotFound,
  submissionNotFound,
  duplicateEmail,
  invalidReviewState,
  notApproved,
  invalidRevenueSplit,
  publisherNotVerified,
  internalError,
} from './errors.js';

// Types
export type {
  AgentListing,
  AgentMetrics,
  AgentPack,
  AgentPrice,
  DomainMetrics,
  EarningTransaction,
  EarningsAccount,
  HumanReview,
  InstallEvent,
  ListingFilter,
  PayoutMethod,
  PayoutRecord,
  Permission,
  PublisherId,
  PublisherProfile,
  ReviewSubmission,
  RevenueSplit,
  ScoredAgent,
  SecurityCheck,
  UninstallEvent,
} from './types.js';

export {
  DeveloperType,
  DeviceType,
  ListingSort,
  ListingStatus,
  ModelTier,
  ReviewDecision,
  ReviewStatus,
  SecurityCheckType,
  Severity,
  createAgentPack,
  createPermission,
  isSensitivePermission,
  isValidSplit,
  newPublisherId,
  priceAmountCents,
  severityGte,
  standardSplit,
  celebritySplit,
} from './types.js';
