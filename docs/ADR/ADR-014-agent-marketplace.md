# ADR-014: Agent Marketplace

| Field    | Value                     |
|----------|---------------------------|
| Status   | Implemented               |
| Date     | 2026-03-19                |
| Authors  | Cedric                    |
| Replaces | —                         |

## Context

RLMX agents are currently first-party only, defined in `rlmx-agents` with 12 typed roles and a static permission matrix (ADR-005). The PRD envisions a marketplace with 10,000+ third-party agents across 12 life domains (Finance, Health, Legal, Career, Education, Home, Shopping, Travel, Social, Government, Automotive, Pet), packaged as RVF containers, with a developer program, revenue sharing, and automated security auditing.

## Decision

### Introduce `rlmx-marketplace` crate

New crate `crates/rlmx-marketplace/` with:

- `Marketplace` struct: top-level orchestrator
- `AgentRegistry`: catalog of all published agents with metadata, ratings, install counts
- `BillingEngine`: Stripe Connect integration, 70/30 revenue split (developer/platform), monthly payouts at $50 threshold
- `ReviewSystem`: automated security audit pipeline (capability analysis, data flow verification, fuzzing) + human review for flagged agents
- `PublisherPortal`: developer dashboard with analytics, earnings, version management
- `FeaturedEngine`: ML-ranked featured/recommended agents based on user domain needs and popularity
- `MarketplaceAnalytics`: install counts, retention, revenue per agent, developer metrics

### AgentListing structure

```rust
pub struct AgentListing {
    id: Uuid,
    name: String,
    domain: LifeDomain,           // 1 of 12 categories
    publisher: PublisherId,
    rvf_hash: [u8; 32],          // Content-addressed RVF container
    price: AgentPrice,            // Free, OneTime($), Monthly($)
    rating: f32,                  // 0-5, community-rated
    install_count: u64,
    permissions_required: Vec<Permission>,
    supported_devices: Vec<DeviceType>,
    min_model_tier: ModelTier,
    size_bytes: u64,
}
```

### 12 Life-Domain Categories

Finance, Health, Legal, Career, Education, Home, Shopping, Travel, Social, Government, Automotive, Pet

### Built on RVF Container Format

Every agent packaged as RVF container (from existing `rlmx-rvf`):

- `SegmentType::Code` -- WASM bytecode
- `SegmentType::Model` -- Quantized model weights (LoRA deltas or full small model)
- `SegmentType::Config` -- Configuration, permissions, UI templates
- `SegmentType::Witness` -- Ed25519 signed proof chain (publisher + marketplace co-signature)

### 8 New MCP Tools

Added to `rlmx-mcp`:

1. `rlmx_marketplace_search` -- Search by domain, keyword, rating, price
2. `rlmx_marketplace_install` -- Download and install RVF container
3. `rlmx_marketplace_uninstall` -- Remove agent and clean up SONA state
4. `rlmx_marketplace_rate` -- Rate and review
5. `rlmx_marketplace_publish` -- Submit agent for review and publication
6. `rlmx_marketplace_earnings` -- Developer earnings dashboard
7. `rlmx_marketplace_featured` -- Get featured/recommended agents
8. `rlmx_marketplace_categories` -- List all 12 life-domain categories

### Developer SDK (`rlmx-agent-sdk`)

- Kernel syscall bindings for agent development
- WASM + native build targets
- Test harness with simulated user interactions
- SONA integration for personalization
- Template agents for each of the 12 domains
- `rlmx-agent` CLI tool: `new`, `dev` (hot-reload), `test` (sandboxed), `publish`

### Agent Install Flow

1. `rvf-runtime` downloads RVF container from marketplace
2. `rvf-crypto` verifies publisher Ed25519 signature + marketplace co-signature
3. `ruvix-cap` derives capability token from user's root token with domain-scoped caveats
4. `ruvector-cognitive-container` creates isolated container (own SONA, own memory, own vector namespace)
5. `ruv-swarm-core` agent discovers existing agents via gossip
6. `TinyDancerRouter` learns new routing weights via online SGD
7. Agent is live -- begins learning user patterns via SONA

### Security Review Pipeline

Automated checks before listing:

- Capability analysis: does the agent request minimal permissions?
- Data flow verification: does data stay within declared scope?
- Fuzzing: 1000+ random inputs for crash detection
- Static analysis: no network calls outside declared NetworkPolicy
- Human review: triggered for agents requesting sensitive permissions (health, finance)

### Celebrity/Influencer Agent Packs

Curated configurations with 50/30/20 revenue split (creator/platform/base agent developer).

### Scale Targets

| Timeframe | First-Party | Third-Party | Developers |
|-----------|-------------|-------------|------------|
| Launch    | 50          | 0           | 0          |
| Year 1    | 100         | 200         | 500        |
| Year 2    | 150         | 1,000       | 2,500      |
| Year 5    | 250         | 10,000+     | 25,000+    |

## Consequences

### Positive

- Creates App Store-level ecosystem moat with double-sided network effects
- RVF container format provides security isolation and portability
- Revenue sharing incentivizes developer ecosystem growth
- Automated review pipeline scales without proportional human review cost

### Negative

- Marketplace abuse (malicious agents, fake reviews) requires ongoing moderation
- Revenue share processing adds financial compliance burden (tax reporting, international payouts)
- Agent quality variance could harm platform reputation

### Risks

- Cold start: marketplace needs 50+ quality agents before users see value
- Developer adoption depends on SDK quality and documentation
- App store policies (Apple 30% cut) may stack with platform 30% cut

## References

- ADR-005: Capability-Secured Agents (permission matrix)
- ADR-011: Sandbox Orchestration (SandboxProfile for agent isolation)
- ADR-010: MCP Tool Expansion (adding 8 new tools)
- PRD Section 6: Agent Marketplace

## Implementation Notes

Implemented in `crates/rlmx-marketplace/` with 9 source files and 38 tests. Marketplace aggregate root. AgentRegistry with CRUD, search, filter by domain/rating/price. BillingEngine (70/30 revenue split, $50 payout threshold). ReviewPipeline with 5 automated security checks. PublisherPortal with verification. FeaturedEngine with ML-ranked scoring. MarketplaceAnalytics. 12 life domains. 8 MCP tools added. CLI: `cargo run -p rlmx-cli -- marketplace search|install|list|featured|publish`
