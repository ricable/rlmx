/**
 * Domain types for the marketplace bounded context (DDD-010, ADR-014).
 * Mirrors rlmx-marketplace/src/domain.rs.
 */

import type { LifeDomain } from '@aix/shared';

// Re-export LifeDomain for convenience within the marketplace package.
export type { LifeDomain } from '@aix/shared';

// ---------------------------------------------------------------------------
// Device & Model types
// ---------------------------------------------------------------------------

/** Device types supported by marketplace agents. */
export enum DeviceType {
  Desktop = 'Desktop',
  Mobile = 'Mobile',
  Tablet = 'Tablet',
  Browser = 'Browser',
  Edge = 'Edge',
  Wearable = 'Wearable',
}

/** Model tier requirements for agents. */
export enum ModelTier {
  Small = 'Small',
  ClaudeCode = 'ClaudeCode',
  Medium = 'Medium',
  Custom = 'Custom',
}

// ---------------------------------------------------------------------------
// Permission
// ---------------------------------------------------------------------------

/**
 * Permission declarations for agent listings.
 * Maps conceptually to kernel SyscallPermission but expressed as strings
 * in the marketplace context to allow extensibility.
 */
export interface Permission {
  readonly name: string;
}

/** Sensitive permission names that escalate to human review. */
const SENSITIVE_PERMISSIONS = new Set([
  'health_data',
  'finance_data',
  'legal_data',
]);

/** Create a Permission with the given name. */
export function createPermission(name: string): Permission {
  return { name };
}

/** Returns true if this permission touches sensitive domains. */
export function isSensitivePermission(permission: Permission): boolean {
  return SENSITIVE_PERMISSIONS.has(permission.name);
}

// ---------------------------------------------------------------------------
// Publisher ID
// ---------------------------------------------------------------------------

/** Publisher identifier (branded string for type safety). */
export type PublisherId = string & { readonly __brand: 'PublisherId' };

/** Generate a new unique PublisherId. */
export function newPublisherId(): PublisherId {
  return crypto.randomUUID() as PublisherId;
}

// ---------------------------------------------------------------------------
// Developer type
// ---------------------------------------------------------------------------

/** Developer type classification. */
export enum DeveloperType {
  Individual = 'Individual',
  Organization = 'Organization',
  Celebrity = 'Celebrity',
}

// ---------------------------------------------------------------------------
// Severity
// ---------------------------------------------------------------------------

/** Severity levels for security checks. */
export enum Severity {
  Low = 'Low',
  Medium = 'Medium',
  High = 'High',
  Critical = 'Critical',
}

/** Numeric weight for severity comparison. */
const SEVERITY_ORDER: Record<Severity, number> = {
  [Severity.Low]: 0,
  [Severity.Medium]: 1,
  [Severity.High]: 2,
  [Severity.Critical]: 3,
};

/** Returns true if a >= b in severity ordering. */
export function severityGte(a: Severity, b: Severity): boolean {
  return SEVERITY_ORDER[a] >= SEVERITY_ORDER[b];
}

// ---------------------------------------------------------------------------
// Review & Listing status
// ---------------------------------------------------------------------------

/** Review status for agent submissions. */
export enum ReviewStatus {
  Pending = 'Pending',
  InReview = 'InReview',
  AutoPassed = 'AutoPassed',
  FlaggedForHuman = 'FlaggedForHuman',
  Approved = 'Approved',
  Rejected = 'Rejected',
}

/** Listing status for agents in the registry. */
export enum ListingStatus {
  Draft = 'Draft',
  InReview = 'InReview',
  Published = 'Published',
  Suspended = 'Suspended',
}

// ---------------------------------------------------------------------------
// Agent pricing
// ---------------------------------------------------------------------------

/** Price model for agents. */
export type AgentPrice =
  | { type: 'Free' }
  | { type: 'OneTime'; cents: number }
  | { type: 'Monthly'; cents: number };

/** Returns the price in cents (0 for free). */
export function priceAmountCents(price: AgentPrice): number {
  switch (price.type) {
    case 'Free':
      return 0;
    case 'OneTime':
    case 'Monthly':
      return price.cents;
  }
}

// ---------------------------------------------------------------------------
// Payout method
// ---------------------------------------------------------------------------

/** Payout method for publisher earnings. */
export type PayoutMethod =
  | { type: 'StripeConnect'; accountId: string }
  | { type: 'BankTransfer'; routing: string; account: string };

// ---------------------------------------------------------------------------
// Revenue split
// ---------------------------------------------------------------------------

/** Revenue split configuration. */
export interface RevenueSplit {
  readonly creatorPct: number;
  readonly platformPct: number;
  readonly baseDevPct: number;
}

/** Standard 70/30 developer/platform split. */
export function standardSplit(): RevenueSplit {
  return { creatorPct: 70, platformPct: 30, baseDevPct: 0 };
}

/** Celebrity/influencer 50/30/20 split. */
export function celebritySplit(): RevenueSplit {
  return { creatorPct: 50, platformPct: 30, baseDevPct: 20 };
}

/** Validates that percentages sum to 100. */
export function isValidSplit(split: RevenueSplit): boolean {
  return split.creatorPct + split.platformPct + split.baseDevPct === 100;
}

// ---------------------------------------------------------------------------
// Agent listing
// ---------------------------------------------------------------------------

/** A published agent listing in the marketplace. */
export interface AgentListing {
  readonly id: string;
  name: string;
  description: string;
  domain: LifeDomain;
  publisher: PublisherId;
  rvfHash: string;
  version: string;
  price: AgentPrice;
  rating: number;
  ratingCount: number;
  installCount: number;
  permissionsRequired: Permission[];
  supportedDevices: DeviceType[];
  minModelTier: ModelTier;
  sizeBytes: number;
  status: ListingStatus;
  publishedAt: string | null;
  createdAt: string;
}

// ---------------------------------------------------------------------------
// Listing filter & sort
// ---------------------------------------------------------------------------

/** Search/filter criteria for querying the registry. */
export interface ListingFilter {
  domain?: LifeDomain;
  keyword?: string;
  minRating?: number;
  maxPriceCents?: number;
  status?: ListingStatus;
  publisher?: PublisherId;
  device?: DeviceType;
}

/** Sort order for listing search results. */
export enum ListingSort {
  Rating = 'Rating',
  Installs = 'Installs',
  Newest = 'Newest',
  PriceLow = 'PriceLow',
  PriceHigh = 'PriceHigh',
}

// ---------------------------------------------------------------------------
// Security check types
// ---------------------------------------------------------------------------

/** Security check types for the review pipeline. */
export enum SecurityCheckType {
  CapabilityMinimality = 'CapabilityMinimality',
  DataFlowVerification = 'DataFlowVerification',
  FuzzTesting = 'FuzzTesting',
  NetworkPolicyCompliance = 'NetworkPolicyCompliance',
  MalwareSignatureScan = 'MalwareSignatureScan',
}

/** Result of a single security check. */
export interface SecurityCheck {
  checkType: SecurityCheckType;
  passed: boolean;
  details: string;
  severity: Severity;
}

// ---------------------------------------------------------------------------
// Agent pack
// ---------------------------------------------------------------------------

/** Celebrity/Influencer agent pack (curated bundle of agents). */
export interface AgentPack {
  readonly id: string;
  name: string;
  creator: PublisherId;
  agents: string[];
  price: AgentPrice;
  revenueSplit: RevenueSplit;
  description: string;
}

/** Create a new agent pack with the celebrity revenue split. */
export function createAgentPack(
  name: string,
  creator: PublisherId,
  agents: string[],
  price: AgentPrice,
  description: string,
  split?: RevenueSplit,
): AgentPack {
  return {
    id: crypto.randomUUID(),
    name,
    creator,
    agents,
    price,
    revenueSplit: split ?? celebritySplit(),
    description,
  };
}

// ---------------------------------------------------------------------------
// Publisher
// ---------------------------------------------------------------------------

/** A registered publisher in the marketplace. */
export interface PublisherProfile {
  readonly id: PublisherId;
  name: string;
  email: string;
  developerType: DeveloperType;
  verified: boolean;
  agents: string[];
  reputationScore: number;
  joinedAt: string;
}

// ---------------------------------------------------------------------------
// Review submission
// ---------------------------------------------------------------------------

/** Human review decision and notes. */
export interface HumanReview {
  reviewerId: string;
  decision: ReviewDecision;
  notes: string;
  reviewedAt: string;
}

/** Decision made by a human reviewer. */
export enum ReviewDecision {
  Approve = 'Approve',
  Reject = 'Reject',
}

/** A review submission tracking entry. */
export interface ReviewSubmission {
  readonly submissionId: string;
  agentId: string;
  automatedChecks: SecurityCheck[];
  humanReview: HumanReview | null;
  status: ReviewStatus;
  submittedAt: string;
  completedAt: string | null;
}

// ---------------------------------------------------------------------------
// Billing types
// ---------------------------------------------------------------------------

/** A single earning transaction. */
export interface EarningTransaction {
  readonly id: string;
  agentId: string;
  userId: string;
  grossCents: number;
  developerCents: number;
  platformCents: number;
  timestamp: string;
}

/** Earnings account for a publisher. */
export interface EarningsAccount {
  publisherId: PublisherId;
  balanceCents: number;
  lifetimeEarnedCents: number;
  payoutMethod: PayoutMethod | null;
  payoutThresholdCents: number;
  transactions: EarningTransaction[];
}

/** A payout record. */
export interface PayoutRecord {
  readonly id: string;
  publisherId: PublisherId;
  amountCents: number;
  processedAt: string;
}

// ---------------------------------------------------------------------------
// Analytics types
// ---------------------------------------------------------------------------

/** A single install event for analytics. */
export interface InstallEvent {
  agentId: string;
  userId: string;
  domain: LifeDomain;
  timestamp: string;
}

/** An uninstall event for retention tracking. */
export interface UninstallEvent {
  agentId: string;
  userId: string;
  timestamp: string;
}

/** Per-agent analytics summary. */
export interface AgentMetrics {
  totalInstalls: number;
  totalUninstalls: number;
  revenueCents: number;
}

/** Per-domain analytics summary. */
export interface DomainMetrics {
  totalInstalls: number;
  totalRevenueCents: number;
  agentCount: number;
}

// ---------------------------------------------------------------------------
// Scored agent (for featured engine)
// ---------------------------------------------------------------------------

/** A scored agent for ranking. */
export interface ScoredAgent {
  agentId: string;
  score: number;
}
