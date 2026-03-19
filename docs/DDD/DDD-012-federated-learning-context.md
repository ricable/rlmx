# DDD-012: Federated Learning Bounded Context

## Overview
The Federated Learning context manages the lifecycle of anonymized pattern sharing across users — from on-device pattern extraction and anonymization, through cloud aggregation, to federated update distribution. This context enforces all privacy invariants and enables collective intelligence without compromising individual privacy.

## Aggregate Root: FederationCycle

### Identity
- `CycleId` — Unique identifier for each weekly federation round
- `epoch: u64` — Monotonically increasing cycle number

### Entities
- **PatternContribution**: An anonymized set of patterns from a single user
  - `contribution_id: ContributionId`
  - `pseudonymous_key: PublicKey` (not linkable to user identity)
  - `domain: LifeDomain` (one of 12 life domains)
  - `pattern_count: u32`
  - `aggregate_quality: f32`
  - `region_bucket: RegionBucket` (coarse geographic, e.g., "US-West")
  - `demographic_bucket: DemographicBucket` (age range, not exact)
  - `submitted_at: Timestamp`
  - `signature: Ed25519Signature`

- **AggregatedModel**: Result of aggregating contributions for a domain
  - `domain: LifeDomain`
  - `cycle_id: CycleId`
  - `contributor_count: u32` (must be >= 1000)
  - `lora_delta: LoRaDelta` (rank-4 aggregated LoRA)
  - `pattern_bank: PatternBank` (top-10K patterns by quality)
  - `router_weights: RouterWeights` (updated TinyDancerRouter config)
  - `created_at: Timestamp`

- **FederatedPackage**: Distributable RVF container with aggregated updates
  - `package_id: PackageId`
  - `domain: LifeDomain`
  - `cycle_id: CycleId`
  - `segments: Vec<RvfSegment>` (Overlay + Pattern + Config)
  - `size_bytes: u64`
  - `checksum: Blake3Hash`

### Value Objects
- `AnonymizationConfig` — Privacy parameters (epsilon, bucketing levels, aggregation threshold)
- `LoRaDelta` — Rank-4 LoRA weight delta (small, efficient to transmit)
- `RegionBucket` — Coarse geographic bucket (country + region, never city)
- `DemographicBucket` — Age range + broad category (never exact demographics)
- `ContributionProof` — Cryptographic proof that contribution was properly anonymized

### Invariants
1. **Aggregation threshold**: No federated pattern published until contributor_count >= 1000 for that domain
2. **Differential privacy**: Laplace noise ε=1.0 applied to all numeric fields BEFORE leaving device
3. **No PII in transit**: Patterns contain ONLY domain, strategy, outcome, region_bucket, demographic_bucket
4. **Pseudonymous keys**: Contribution keys are NOT the user's identity key — unlinkable
5. **No speaker embeddings**: Voice patterns federated as text features only
6. **Emotion bucketing**: Emotion valence reduced to 5 discrete levels (very negative / negative / neutral / positive / very positive)
7. **One-way flow**: Raw user data flows only from user → anonymization → cloud. Cloud NEVER sends data back that could deanonymize.
8. **EWC++ preservation**: Federated LoRA application must not overwrite personal learned patterns

## Domain Events
| Event | Trigger | Consumers |
|-------|---------|-----------|
| `ContributionPrepared` | User's home hub finishes anonymizing weekly patterns | Upload scheduler |
| `ContributionReceived` | Cloud registry receives a valid signed contribution | AggregatedModel builder |
| `AggregationCompleted` | Domain reaches threshold and models are trained | FederatedPackage builder |
| `PackagePublished` | New federated update ready for distribution | All user devices (pull) |
| `PackageApplied` | User device applies federated update | SONA, TinyDancerRouter |
| `ContributionRejected` | Signature invalid or anomaly detected | Security audit log |

## Commands
- `PrepareContribution(user_patterns, anonymization_config)` → Anonymize and package patterns
- `SubmitContribution(contribution, signature)` → Upload to cloud registry
- `AggregateContributions(domain, cycle_id)` → Train aggregated model from all contributions
- `PublishPackage(aggregated_model)` → Create and distribute RVF federated package
- `ApplyPackage(package, local_sona)` → Import federated updates with EWC++ preservation
- `OptOut(user_id)` → Stop contributing (still receives federated updates)

## Queries
- `GetLatestPackage(domain)` → Most recent federated update for a domain
- `GetCycleStatus(cycle_id)` → Contribution counts, aggregation status per domain
- `GetContributionHistory(pseudonymous_key)` → User's contribution log (local only)
- `GetDomainCoverage()` → Which domains have enough contributors for federation

## Anti-Corruption Layer
- **To Observation & Health Context (DDD-006)**: Reads SONA patterns for contribution, writes federated updates back
- **To Container & Storage Context (DDD-007)**: Uses RVF container format for packaging
- **To Agent Marketplace Context (DDD-010)**: New marketplace agents bootstrap from federated packages
- **To Kernel Syscall Context (DDD-001)**: TinyDancerRouter weight updates flow through kernel

## Crate Mapping
- Primary: Extension of `rlmx-cognitive` (anonymization, SONA integration) + new federation service in `rlmx-swarm` or dedicated `rlmx-federation`
- Dependencies: `rlmx-cognitive` (SONA, FederatedAnonymizer), `rlmx-rvf` (container format), `rvf-crypto` (signing), `ruvector-gnn` (aggregate GNN training), `ruvector-sona` (pattern merge)

## Ubiquitous Language Additions
- **Lift**: A single agent interaction that saves the user time, money, health risk, or cognitive load
- **Pattern Contribution**: Anonymized (input, strategy, outcome) triple shared for federation
- **Federated Package**: RVF container containing aggregated LoRA deltas + pattern bank + router weights
- **Privacy Anchor**: The home hub device that owns all raw personal data
- **Bootstrap**: Process of making a new agent immediately competent via federated patterns
