# ADR-025: Subscription & Billing Tiers

Status: Implemented

## Context
RuVix Mesh monetizes through a tiered subscription model. The billing system must enforce agent limits, cloud burst access, federation participation, and family sharing while keeping the core agent runtime on-device and functional even without payment.

## Decision
Implement a 6-tier subscription model with capability-token-enforced limits, integrated with the existing RBAC system and marketplace billing engine.

### Tier Definitions

| Tier | Price | Agent Limit | Cloud Burst | Federation | Family | Marketplace | Audit Trail |
|------|-------|-------------|-------------|------------|--------|-------------|-------------|
| Free | $0 | 5 agents | None | Community (receive only) | No | Browse only | None |
| Personal | $9.99/mo | Unlimited | Yes | Full (send + receive) | No | Install + rate | Basic |
| Family | $19.99/mo | Unlimited | Yes | Full | 6 members | Install + rate | Basic |
| Pro | $29.99/mo | Unlimited | Priority | Full + early access | No | Install + create SDK | Witness chain |
| Enterprise | Custom | Unlimited | Dedicated | Private federation | Org-wide | Custom agents | SOC2/HIPAA compliant |
| Developer | Free + 70/30 | Unlimited | Yes | Full + training data access | No | Publish + revenue share | Publisher analytics |

### Enforcement via Capability Tokens
Subscription tier is encoded in the user's root capability token as a Macaroon caveat:
```
caveat: tier = "personal"
caveat: agent_limit = unlimited
caveat: cloud_burst = true
caveat: federation_mode = "full"
caveat: expires = "2026-04-19T00:00:00Z"
```

When an agent attempts a tier-restricted operation:
1. `ruvix-cap` validates the capability token's tier caveat
2. If the operation exceeds tier limits (e.g., Free user spawning 6th agent), syscall returns `PermissionDenied`
3. Agent runtime surfaces upgrade prompt to user

### Free Tier Agent Allocation
5 default agents (non-removable, non-replaceable on Free):
1. Email Triage (Router agent, Zone D)
2. Calendar Manager (Worker agent, Zone A/D)
3. Weather & Commute (Monitor agent, Zone D)
4. News Digest (Analyst agent, Zone D)
5. Basic Shopping (Monitor agent, Zone D)

### Family Plan Architecture
- Family owner creates family group, invites up to 5 additional members
- Each member gets their own root capability token derived from family token
- Shared family agents (grocery, chores, budget) run on family owner's home hub
- Per-member privacy boundaries enforced by capability token caveats
- `ruvector-raft`: Family Raft cluster for shared state consistency
- Cross-member agent communication requires explicit `family_share` permission

### Cloud Burst Billing
- Metered by inference tokens consumed on Zone B (cloud)
- Free: 0 tokens (device-only inference)
- Personal/Family: 100K tokens/month included, $0.001/token overage
- Pro: 500K tokens/month included, $0.0008/token overage
- Enterprise: Dedicated pool, negotiated pricing

### Marketplace Revenue Share
- Agent sales: 70% to developer, 30% to platform (per ADR-014)
- Monthly payouts at $50 threshold (existing `BillingEngine`)
- Developer tier: free platform access in exchange for publishing agents
- Pro tier: SDK access for creating custom agents (private, not published)

### Integration Points
- `rlmx-marketplace/src/billing.rs`: Existing BillingEngine extended with subscription tiers
- `rlmx-kernel`: Tier validation at syscall dispatch (via capability token)
- `rlmx-mcp/src/server.rs`: RBAC roles mapped to subscription tiers
- `rlmx-phone/src/engagement.rs`: Tier-aware achievement system

### Payment Processing
- Stripe integration for subscription management (external, not in kernel)
- Webhook handler updates capability token on subscription change
- Grace period: 7 days after payment failure before downgrade
- Downgrade: excess agents suspended (not deleted), resume on re-subscription

## Consequences

### Positive
- Free tier provides real value (5 useful agents) — drives adoption
- Capability tokens enforce limits at the kernel level — no bypass possible
- Bill Negotiator alone saves $847/year — Personal tier pays for itself 7x
- Family plan enables shared agents with privacy boundaries
- Developer tier creates marketplace ecosystem

### Negative
- Tier enforcement adds latency to every syscall (token validation)
- Downgrade UX is painful (suspended agents)
- Free tier agents are fixed — no customization until upgrade

### Risks
- Price sensitivity in non-US markets (mitigated: regional pricing planned)
- Free tier too generous → low conversion (mitigated: 5 agents is useful but limited)
- Free tier too restrictive → no adoption (mitigated: core agents are genuinely useful)

## References
- PRD: "RuVix Mesh", Monetization section
- ADR-005: Capability-Secured Agents
- ADR-014: Agent Marketplace (billing engine, revenue share)
- ADR-016: Engagement & Gamification
