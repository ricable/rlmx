# ADR-023: Federated Learning Pipeline

Status: Proposed

## Context
RuVix Mesh's core value proposition is compound intelligence — agents that learn from individual user interactions AND from anonymized patterns across millions of users. This requires a federated learning pipeline that preserves privacy while enabling collective improvement.

## Decision
Implement a weekly federated learning cycle with on-device anonymization, cloud aggregation, and bidirectional pattern distribution using existing RVF container format and SONA infrastructure.

### Per-User Learning (Private, On-Device)

#### Continuous Learning Loop
1. Every agent interaction records `(query_embedding, actions_taken, result_quality)` via `ruvector-sona`
2. When patterns exceed 100 per domain: `MicroLoRA` computes rank-4 LoRA delta
3. `FisherInformation` updates EWC++ diagonal to prevent catastrophic forgetting
4. Delta applied to local model weights — agent personalized to THIS user

#### Daily Consolidation (Home Hub, Idle)
- `TinyDancerRouter`: Retrain from routing history (which queries went where, latency, quality)
- `DagOptimizer`: Update strategy success rates across multi-agent decisions
- `NervousSystem`: Circadian adjustment — schedule heavy inference during user's active hours
- `cognitum-rs`: Meta-learning — identify stale patterns, agents needing retraining

### Federated Cycle (Weekly, Anonymous)

#### Step 1: On-Device Anonymization
- `FederatedAnonymizer` (existing in `rlmx-cognitive`):
  - Strip all PII from patterns
  - Emotion valence bucketed to 5 levels (per ADR-017)
  - Laplace noise ε=1.0 on urgency/satisfaction scores
  - No speaker embeddings federated
  - Minimum 1000-user aggregation threshold before any pattern published
- Output: `{domain, strategy, outcome_quality, region_bucket, demographic_bucket}`

#### Step 2: Package & Upload
- `rvf-types`: Package as `SegmentType::TransferPrior`
- `rvf-crypto`: Sign with user's pseudonymous contribution key (not user identity key)
- `rvf-wire`: Serialize for transport
- `midstreamer-quic` (LAN) or HTTPS (WAN): Upload to Zone B RVF Registry

#### Step 3: Cloud Aggregation (Zone B)
- `rvf-index`: Index incoming patterns by domain, region, demographic bucket
- `ruvector-gnn`: Train aggregate Graph Neural Network on 1M+ anonymized patterns
- `ruvector-sona`: Merge patterns into federated PatternBank (top-10K per domain, ranked by outcome quality)
- Compute aggregated LoRA deltas per domain (average of contributing users' deltas, weighted by outcome quality)

#### Step 4: Distribution
Produce federated update packages:
- `SegmentType::Overlay` — Aggregated LoRA deltas per domain
- `SegmentType::Pattern` — Merged pattern bank (top-10K per domain)
- `SegmentType::Config` — Updated TinyDancerRouter weights

Distribute to all users' devices via RVF container download.

#### Step 5: On-Device Application
- `ruvector-sona`: Import federated patterns with EWC++ preservation
- Local knowledge preserved, global knowledge incorporated
- `MicroLoRA`: Apply aggregated LoRA delta with regularization against local delta

### Privacy Invariants (Non-Negotiable)
1. No raw user data ever leaves the device
2. Patterns anonymized BEFORE leaving device, not after
3. Pseudonymous contribution keys — cloud cannot link patterns to user identity
4. Differential privacy: Laplace noise ε=1.0 on all numeric fields
5. Minimum aggregation threshold: 1000 users before any federated pattern is published
6. No speaker embeddings, no location coordinates, no names in federated data
7. User can opt out of federation entirely — agents still learn from personal data

### New User Bootstrap
When a new user installs an agent:
1. Download latest federated pattern pack for that domain
2. Import 10K+ anonymized patterns into agent's SONA bank
3. Apply aggregated LoRA delta — agent is immediately competent
4. Zero personal history needed — collective intelligence bootstraps the agent

## Consequences

### Positive
- New users get instant agent competence from collective intelligence
- Continuous improvement across millions of users benefits everyone
- Privacy preserved through anonymization + differential privacy + aggregation thresholds
- Uses existing RVF container format — no new wire protocol needed
- EWC++ ensures personal knowledge not overwritten by federated updates

### Negative
- Weekly cycle means new patterns take up to 7 days to propagate
- Cloud aggregation requires compute resources (NUC cluster)
- Aggregated LoRA deltas may not transfer well across very different user profiles
- Users who opt out of federation still benefit from others' contributions (free rider)

### Risks
- Privacy attack via pattern intersection (mitigated: aggregation threshold + differential privacy)
- Model poisoning via malicious pattern injection (mitigated: crypto signature verification + outlier detection)
- Federation bandwidth on metered connections (mitigated: delta compression, Wi-Fi-only default)

## References
- PRD: "RuVix Mesh", Federated Learning sections
- ADR-017: Federated Voice Learning (privacy invariants)
- ADR-006: Evolutionary Auto-Research (SONA learning loop)
- DDD-006: Observation & Health context (SONA)
