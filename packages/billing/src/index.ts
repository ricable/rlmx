/**
 * @aix/billing — Subscription billing bounded context (DDD-013, ADR-025).
 *
 * Manages 6-tier subscriptions, family plans, developer revenue share,
 * usage tracking, and tier-based capability enforcement.
 */

// --- Errors ---
export {
  BillingError,
  type BillingErrorCode,
} from './errors.js';

// --- Events ---
export {
  type SubscriptionStatus,
  type SubscriptionEvent,
} from './events.js';

// --- Tiers (re-exported from @aix/shared + local enforcement) ---
export {
  SubscriptionTier,
  FederationMode,
  monthlyPriceCents,
  tierLimits,
  type TierLimits,
  type TierFeatures,
  type EnforcementViolation,
  type TierFeatureName,
  enforceAgentLimit,
  enforceTokenLimit,
  enforceFeature,
} from './tier.js';

// --- Subscription aggregate ---
export {
  type Subscription,
  GRACE_PERIOD_DAYS,
  subscribe,
  isActive,
  upgrade,
  downgrade,
  cancel,
  reactivate,
  markPastDue,
  suspend,
} from './subscription.js';

// --- Usage metrics ---
export {
  type UsageMetrics,
  createUsageMetrics,
  recordTokenUsage,
  checkAgentQuota,
  recordAgentSpawn,
  recordAgentTerminate,
  resetPeriod,
  updateLimits,
  tokenUtilization,
} from './usage.js';

// --- Family ---
export {
  MAX_FAMILY_MEMBERS,
  type FamilyRole,
  type PrivacyBoundary,
  type FamilyMember,
  type FamilyGroup,
  defaultPrivacyBoundary,
  childPrivacyBoundary,
  createFamilyGroup,
  addMember,
  removeMember,
  shareAgent,
  unshareAgent,
  getMember,
  memberCount,
} from './family.js';

// --- Developer ---
export {
  DEVELOPER_SHARE_PCT,
  DEFAULT_PAYOUT_THRESHOLD_CENTS,
  type PayoutMethod,
  type SaleRecord,
  type PayoutRecord,
  type DeveloperAccount,
  createDeveloperAccount,
  setPayoutMethod,
  recordSale,
  calculatePayout,
  requestPayout,
  publishAgent,
  totalSales,
} from './developer.js';

// --- Capability enforcement ---
export {
  type TierCaveat,
  type TierCapabilityToken,
  TierCapabilityEnforcer,
} from './capability-enforcement.js';

// --- Budget ledger (ADR-032) ---
export {
  type BudgetEntry,
  type BudgetPolicy,
  type BudgetDecision,
  type BudgetReportEntry,
  BudgetLedger,
} from './budget.js';
