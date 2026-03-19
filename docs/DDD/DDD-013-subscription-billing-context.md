# DDD-013: Subscription & Billing Bounded Context

## Overview
The Subscription & Billing context manages user subscription tiers, enforces tier-based limits on agent count and cloud burst usage, handles payment processing, family plan membership, and developer revenue share. It integrates with the capability token system to enforce limits at the kernel level.

## Aggregate Root: Subscription

### Identity
- `SubscriptionId` — Unique subscription identifier
- `UserId` — The subscribing user (or family owner)

### Entities
- **SubscriptionPlan**: The user's active subscription
  - `tier: SubscriptionTier` (Free, Personal, Family, Pro, Enterprise, Developer)
  - `status: SubscriptionStatus` (Active, PastDue, GracePeriod, Suspended, Cancelled)
  - `started_at: Timestamp`
  - `current_period_end: Timestamp`
  - `payment_method: Option<PaymentMethodId>`
  - `stripe_subscription_id: Option<String>`

- **UsageMetrics**: Tracks metered usage within billing period
  - `cloud_tokens_used: u64`
  - `cloud_tokens_limit: u64`
  - `agents_active: u32`
  - `agents_limit: Option<u32>` (None = unlimited)
  - `period_start: Timestamp`
  - `period_end: Timestamp`

- **FamilyGroup**: Family plan shared membership
  - `family_id: FamilyId`
  - `owner_user_id: UserId`
  - `members: Vec<FamilyMember>` (max 6 including owner)
  - `shared_agents: Vec<AgentId>` (grocery, chores, budget)
  - `created_at: Timestamp`

- **FamilyMember**: A member of a family plan
  - `user_id: UserId`
  - `role: FamilyRole` (Owner, Adult, Child)
  - `privacy_boundary: CapabilityToken` (per-member isolation)
  - `joined_at: Timestamp`

- **DeveloperAccount**: Developer tier publishing account
  - `developer_id: DeveloperId`
  - `publisher_name: String`
  - `payout_method: Option<PayoutMethod>`
  - `balance: Decimal` (accumulated revenue share)
  - `payout_threshold: Decimal` ($50 default)
  - `published_agents: Vec<AgentListingId>`

### Value Objects
- `SubscriptionTier` — Enum: Free | Personal | Family | Pro | Enterprise | Developer
- `TierLimits` — Agent count, cloud tokens, federation mode, features per tier
- `CloudBurstQuota` — Tokens included + overage rate per tier
- `RevenueShare` — 70/30 split configuration
- `GracePeriod` — 7 days after payment failure

### Invariants
1. Free tier: exactly 5 agents, no cloud burst, receive-only federation
2. Agent limit enforced at spawn time via capability token tier caveat
3. Cloud burst tokens metered per billing period, overage charged
4. Family plan: max 6 members, owner manages membership
5. Family members have isolated capability tokens — cross-member access requires explicit `family_share` permission
6. Developer revenue share: 70% developer / 30% platform, payout at $50 threshold
7. Grace period: 7 days after payment failure before capabilities downgrade
8. Downgrade: excess agents suspended (state preserved), not deleted
9. Enterprise tier: custom limits negotiated per contract
10. Capability tokens re-derived on tier change — old tokens invalidated

## Domain Events
| Event | Trigger | Consumers |
|-------|---------|-----------|
| `SubscriptionCreated` | User signs up for a tier | CapabilityToken minter, AgentRegistry |
| `SubscriptionUpgraded` | User moves to higher tier | CapabilityToken re-derive (expand limits) |
| `SubscriptionDowngraded` | User moves to lower tier | Agent suspension, token re-derive (restrict) |
| `PaymentFailed` | Stripe payment fails | Grace period timer, user notification |
| `GracePeriodExpired` | 7 days after payment failure | Downgrade to Free, agent suspension |
| `CloudBurstExhausted` | Monthly token quota depleted | TinyDancerRouter (stop cloud routing), user notification |
| `FamilyMemberAdded` | Owner invites new family member | Derive member capability token |
| `FamilyMemberRemoved` | Owner removes member or member leaves | Revoke member token, migrate shared agent state |
| `DeveloperPayoutTriggered` | Balance exceeds $50 threshold | Payout processing |
| `AgentPurchased` | User buys marketplace agent | Developer balance credit, platform revenue |

## Commands
- `CreateSubscription(user_id, tier, payment_method)` — Initialize subscription and mint capability token
- `UpgradeTier(subscription_id, new_tier)` — Expand capabilities, prorate billing
- `DowngradeTier(subscription_id, new_tier)` — Restrict capabilities at period end
- `CancelSubscription(subscription_id)` — Cancel at period end, retain data for 90 days
- `RecordCloudUsage(subscription_id, tokens_consumed)` — Track metered cloud burst
- `CreateFamilyGroup(owner_id)` — Initialize family plan
- `InviteFamilyMember(family_id, email)` — Send invite, derive member token on accept
- `RemoveFamilyMember(family_id, member_id)` — Revoke access, clean up shared state
- `ProcessPayout(developer_id)` — Transfer accumulated balance to developer

## Queries
- `GetSubscription(user_id)` — Current tier, status, usage, limits
- `GetUsageReport(subscription_id, period)` — Detailed usage breakdown
- `GetFamilyMembers(family_id)` — All members and their roles
- `GetDeveloperEarnings(developer_id, period)` — Revenue by agent, total balance

## Anti-Corruption Layer
- **To Kernel Syscall Context (DDD-001)**: Tier limits encoded in capability tokens, validated at syscall dispatch
- **To Agent Lifecycle Context (DDD-002)**: Agent spawn checks tier limit, suspension on downgrade
- **To Agent Marketplace Context (DDD-010)**: Revenue share calculation, developer payouts
- **To Inference Routing Context (DDD-004)**: Cloud burst quota informs TinyDancerRouter routing decisions
- **To Phone Runtime Context (DDD-009)**: Tier-aware engagement achievements

## Crate Mapping
- Primary: New `rlmx-billing` crate or extension of `rlmx-marketplace/src/billing.rs`
- Dependencies: `rlmx-kernel` (capability tokens, types), `rlmx-marketplace` (BillingEngine), `ruvix-cap` (token derivation with tier caveats)
- External: Stripe SDK (payment processing, webhook handling)

## Ubiquitous Language Additions
- **Tier**: Subscription level determining agent limits, cloud access, and features
- **Cloud Burst**: On-demand cloud inference for queries exceeding device capability
- **Grace Period**: 7-day window after payment failure before capability downgrade
- **Family Boundary**: Capability-token-enforced isolation between family members' agent data
- **Revenue Share**: 70/30 split between agent developer and platform on marketplace sales
- **Payout Threshold**: Minimum balance ($50) before developer payout is triggered
