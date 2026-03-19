# DDD-010: Agent Marketplace Bounded Context

## Overview

The Marketplace context manages the agent ecosystem: publishing, discovery,
installation, billing, review, and developer analytics. It is the
**distribution backbone** that connects third-party developers to users,
handling the full lifecycle from agent submission through revenue payout.
It enforces security through automated auditing and maintains trust through
community ratings.

**Crate**: `crates/rlmx-marketplace/` (planned)

## Aggregate Root: Marketplace

```rust
pub struct Marketplace {
    pub registry: AgentRegistry,
    pub billing: BillingEngine,
    pub review_pipeline: ReviewPipeline,
    pub publisher_portal: PublisherPortal,
    pub featured: FeaturedEngine,
    pub analytics: MarketplaceAnalytics,
    pub categories: Vec<LifeDomain>,        // 12 life domains
}
```

`Marketplace` is the aggregate root because it owns the consistency boundary
for all agent listings, publisher accounts, billing state, and review status.
All mutations to marketplace state flow through it.

## Entities

### AgentListing

```rust
pub struct AgentListing {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub domain: LifeDomain,
    pub publisher: PublisherId,
    pub rvf_hash: [u8; 32],              // Content-addressed RVF container
    pub version: SemanticVersion,
    pub price: AgentPrice,
    pub rating: f32,                      // 0.0-5.0
    pub rating_count: u64,
    pub install_count: u64,
    pub permissions_required: Vec<Permission>,
    pub supported_devices: Vec<DeviceType>,
    pub min_model_tier: ModelTier,
    pub size_bytes: u64,
    pub status: ListingStatus,            // Draft, InReview, Published, Suspended
    pub published_at: Option<DateTime<Utc>>,
}
```

### Publisher

```rust
pub struct Publisher {
    pub id: PublisherId,
    pub name: String,
    pub developer_type: DeveloperType,    // Individual, Organization, Celebrity
    pub verified: bool,
    pub agents: Vec<Uuid>,               // Published agent IDs
    pub earnings: EarningsAccount,
    pub joined_at: DateTime<Utc>,
    pub reputation_score: f32,
}
```

### ReviewPipeline

```rust
pub struct ReviewPipeline {
    pub submission_id: Uuid,
    pub agent_id: Uuid,
    pub automated_checks: Vec<SecurityCheck>,
    pub human_review: Option<HumanReview>,
    pub status: ReviewStatus,            // Pending, AutoPassed, FlaggedForHuman, Approved, Rejected
    pub submitted_at: DateTime<Utc>,
}
```

### EarningsAccount

```rust
pub struct EarningsAccount {
    pub publisher_id: PublisherId,
    pub balance_cents: u64,
    pub lifetime_earned_cents: u64,
    pub payout_method: PayoutMethod,     // Stripe Connect
    pub payout_threshold_cents: u64,     // Default 5000 ($50)
    pub next_payout_date: DateTime<Utc>,
    pub transactions: Vec<EarningTransaction>,
}
```

## Value Objects

| Value Object | Definition |
|-------------|------------|
| `PublisherId` | Newtype wrapping `Uuid`. Identifies a publisher account. |
| `ListingStatus` | Enum: `Draft`, `InReview`, `Published`, `Suspended` |
| `ReviewStatus` | Enum: `Pending`, `AutoPassed`, `FlaggedForHuman`, `Approved`, `Rejected` |
| `DeveloperType` | Enum: `Individual`, `Organization`, `Celebrity` |
| `PayoutMethod` | Enum: `StripeConnect(String)`, `BankTransfer(BankDetails)` |
| `Severity` | Enum: `Low`, `Medium`, `High`, `Critical` |
| `SemanticVersion` | `{ major: u32, minor: u32, patch: u32 }` |

### LifeDomain (12 categories)

```rust
pub enum LifeDomain {
    Finance,
    Health,
    Legal,
    Career,
    Education,
    Home,
    Shopping,
    Travel,
    Social,
    Government,
    Automotive,
    Pet,
}
```

### AgentPrice

```rust
pub enum AgentPrice {
    Free,
    OneTime(u64),          // cents
    Monthly(u64),          // cents/month
}
```

### SecurityCheck

```rust
pub struct SecurityCheck {
    pub check_type: SecurityCheckType,
    pub passed: bool,
    pub details: String,
    pub severity: Severity,
}

pub enum SecurityCheckType {
    CapabilityMinimality,     // Requests only needed permissions?
    DataFlowVerification,     // Data stays in declared scope?
    FuzzTesting,              // 1000+ random inputs, no crashes?
    NetworkPolicyCompliance,  // No undeclared network calls?
    MalwareSignatureScan,     // Known malicious patterns?
}
```

### AgentPack (Celebrity/Influencer bundles)

```rust
pub struct AgentPack {
    pub id: Uuid,
    pub name: String,
    pub creator: PublisherId,
    pub agents: Vec<(Uuid, AgentConfig)>,  // Agent ID + custom config
    pub price: AgentPrice,
    pub revenue_split: RevenueSplit,        // 50/30/20 creator/platform/base-dev
}
```

### RevenueSplit

```rust
pub struct RevenueSplit {
    pub creator_pct: u8,     // e.g. 50
    pub platform_pct: u8,    // e.g. 30
    pub base_dev_pct: u8,    // e.g. 20
}
```

## Domain Events

| Event | Trigger | Data |
|-------|---------|------|
| `AgentSubmitted` | Publisher submits a new agent | `{ agent_id, publisher_id }` |
| `ReviewStarted` | Submission enters the review pipeline | `{ agent_id, submission_id }` |
| `ReviewCompleted` | All automated + human checks finished | `{ agent_id, status: ReviewStatus }` |
| `AgentPublished` | Agent passes review and goes live | `{ agent_id, domain: LifeDomain }` |
| `AgentInstalled` | User installs an agent | `{ agent_id, user_id, device: DeviceType }` |
| `AgentUninstalled` | User removes an agent | `{ agent_id, user_id }` |
| `AgentRated` | User submits a rating/review | `{ agent_id, user_id, rating: f32, review: Option<String> }` |
| `AgentSuspended` | Agent pulled from marketplace | `{ agent_id, reason: String }` |
| `PayoutProcessed` | Publisher receives earnings payout | `{ publisher_id, amount_cents: u64 }` |
| `PackCreated` | Celebrity/influencer creates a bundle | `{ pack_id, creator: PublisherId }` |

```rust
pub enum MarketplaceDomainEvent {
    AgentSubmitted { agent_id: Uuid, publisher_id: PublisherId },
    ReviewStarted { agent_id: Uuid, submission_id: Uuid },
    ReviewCompleted { agent_id: Uuid, status: ReviewStatus },
    AgentPublished { agent_id: Uuid, domain: LifeDomain },
    AgentInstalled { agent_id: Uuid, user_id: Uuid, device: DeviceType },
    AgentUninstalled { agent_id: Uuid, user_id: Uuid },
    AgentRated { agent_id: Uuid, user_id: Uuid, rating: f32, review: Option<String> },
    AgentSuspended { agent_id: Uuid, reason: String },
    PayoutProcessed { publisher_id: PublisherId, amount_cents: u64 },
    PackCreated { pack_id: Uuid, creator: PublisherId },
}
```

## Domain Services

### AgentSubmissionService

```rust
impl AgentSubmissionService {
    /// Submit a new agent for review.
    /// 1. Validate RVF container integrity (hash check).
    /// 2. Verify publisher Ed25519 signature on RVF Witness segment.
    /// 3. Extract declared permissions from RVF Config segment.
    /// 4. Create ReviewPipeline entry with Pending status.
    /// 5. Emit AgentSubmitted event.
    pub async fn submit(
        &self,
        publisher_id: PublisherId,
        rvf_container: &[u8],
        listing: AgentListingDraft,
    ) -> Result<Uuid, MarketplaceError>;
}
```

### ReviewService

```rust
impl ReviewService {
    /// Run all 5 automated security checks against the RVF container.
    /// If any check with severity >= High fails, mark as Rejected.
    /// If the agent requests health, finance, or legal permissions,
    /// escalate to FlaggedForHuman.
    /// Otherwise, mark as AutoPassed and proceed to co-signing.
    pub async fn run_automated_review(
        &self,
        submission_id: Uuid,
    ) -> Result<ReviewStatus, MarketplaceError>;

    /// Human reviewer approves or rejects a flagged submission.
    pub async fn complete_human_review(
        &self,
        submission_id: Uuid,
        decision: ReviewDecision,
        notes: String,
    ) -> Result<ReviewStatus, MarketplaceError>;
}
```

### PublishingService

```rust
impl PublishingService {
    /// Publish an approved agent to the marketplace.
    /// 1. Verify ReviewStatus is Approved.
    /// 2. Co-sign the RVF Witness segment with marketplace Ed25519 key.
    /// 3. Store the content-addressed RVF container.
    /// 4. Set ListingStatus to Published.
    /// 5. Emit AgentPublished event.
    pub async fn publish(
        &self,
        agent_id: Uuid,
    ) -> Result<(), MarketplaceError>;
}
```

### BillingService

```rust
impl BillingService {
    /// Record an agent installation and charge the user if applicable.
    /// Credits 70% to the publisher's EarningsAccount.
    pub async fn process_installation(
        &self,
        agent_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), MarketplaceError>;

    /// Process monthly payouts for all publishers above the threshold.
    /// Uses Stripe Connect for actual fund transfer.
    pub async fn run_payout_cycle(&self) -> Result<Vec<PayoutProcessed>, MarketplaceError>;
}
```

## Invariants

1. **Double signature required**: Every published agent must have both
   publisher Ed25519 signature AND marketplace co-signature on its RVF
   Witness segment. Publishing without both signatures is rejected.

2. **Automated review mandatory**: No agent can reach `Published` status
   without passing all 5 `SecurityCheckType` variants. There are no
   bypasses or fast-tracks.

3. **Human review for sensitive permissions**: Agents requesting health,
   finance, or legal data access trigger mandatory human review. The
   `ReviewService` escalates to `FlaggedForHuman` automatically.

4. **Revenue split enforced**: Platform always receives exactly 30%.
   Developer receives 70% (standard) or split per `AgentPack` terms
   where creator + platform + base-dev = 100%.

5. **Payout threshold**: Earnings below $50 (5000 cents) accumulate
   until threshold is met. Monthly payout cycle. No manual override.

6. **Rating minimum**: Agents with rating below 2.0 after 50+ ratings
   are auto-suspended and enter re-review. Suspension emits
   `AgentSuspended`.

7. **Version immutability**: Published RVF containers are
   content-addressed (`rvf_hash`). Updates create new versions with
   new hashes; existing containers are never modified.

8. **Permission escalation forbidden**: Agent updates cannot request
   broader permissions than the original version without triggering a
   full re-review through the `ReviewPipeline`.

## Context Relationships

| Related Context | Relationship | Description |
|----------------|-------------|-------------|
| Kernel Syscall (DDD-002) | Upstream | Agents call kernel syscalls; marketplace does not own syscall definitions |
| Agent Lifecycle (DDD-003) | Partner | Marketplace publishes agents that the lifecycle context manages at runtime |
| Swarm Coordination (DDD-004) | Upstream | Installed agents join swarms; marketplace does not manage swarm topology |
| Inference Routing (DDD-005) | Upstream | `min_model_tier` on listings maps to routing context tiers; marketplace does not route inference |

## Anti-Corruption Layer

The marketplace context maintains strict isolation from adjacent contexts:

- **No agent execution**: Marketplace never executes agent code. It only
  stores, reviews, and distributes RVF containers. Execution is the
  responsibility of the Agent Lifecycle context (DDD-003).

- **Permission declaration only**: Agent permissions are declared in the
  RVF Config segment. Actual enforcement happens in the kernel capability
  system (DDD-002). Marketplace validates declarations for completeness
  but does not enforce at runtime.

- **Billing isolation**: Billing is isolated behind Stripe Connect.
  Marketplace stores transaction references (`PayoutMethod`,
  `EarningTransaction`), never raw payment credentials. PCI compliance
  is delegated entirely to Stripe.

- **Reputation boundary**: Publisher `reputation_score` is computed from
  marketplace-internal signals only (ratings, install count, review pass
  rate). External reputation sources (social media, other platforms) are
  never imported.

- **ModelTier mapping**: `min_model_tier` on listings uses the same
  `ModelTier` enum from `rlmx-kernel`, imported via the shared types
  crate. Marketplace does not define its own tier hierarchy.

## File Map (planned)

| File | Types |
|------|-------|
| `crates/rlmx-marketplace/src/listing.rs` | `AgentListing`, `ListingStatus`, `AgentPrice`, `SemanticVersion` |
| `crates/rlmx-marketplace/src/publisher.rs` | `Publisher`, `PublisherId`, `DeveloperType`, `EarningsAccount` |
| `crates/rlmx-marketplace/src/review.rs` | `ReviewPipeline`, `ReviewStatus`, `SecurityCheck`, `SecurityCheckType` |
| `crates/rlmx-marketplace/src/billing.rs` | `BillingEngine`, `BillingService`, `PayoutMethod`, `EarningTransaction` |
| `crates/rlmx-marketplace/src/pack.rs` | `AgentPack`, `RevenueSplit` |
| `crates/rlmx-marketplace/src/featured.rs` | `FeaturedEngine`, ranking and promotion logic |
| `crates/rlmx-marketplace/src/analytics.rs` | `MarketplaceAnalytics`, install trends, revenue reports |
| `crates/rlmx-marketplace/src/domain.rs` | `LifeDomain` enum, `MarketplaceDomainEvent` |
| `crates/rlmx-marketplace/src/submission.rs` | `AgentSubmissionService`, RVF validation |
| `crates/rlmx-marketplace/src/lib.rs` | Public re-exports |

## Implementation Status

**Status**: Implemented in `crates/rlmx-marketplace/`

| Module | File | Tests |
|--------|------|-------|
| Marketplace (aggregate root) | `src/marketplace.rs` | — |
| AgentRegistry | `src/registry.rs` | 12 |
| BillingEngine | `src/billing.rs` | 6 |
| ReviewPipeline | `src/review.rs` | 5 |
| PublisherPortal | `src/publisher.rs` | 5 |
| FeaturedEngine | `src/featured.rs` | 5 |
| MarketplaceAnalytics | `src/analytics.rs` | 5 |
| Domain types | `src/domain.rs` | — |

**Total**: 38 tests, all passing.

8 MCP tools added to `crates/rlmx-mcp/src/tools.rs` for marketplace operations.
Mobile marketplace UI in `mobile/src/screens/MarketplaceScreen.tsx` with 25 demo agents across 12 domains.
