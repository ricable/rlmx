# ADR-015: Multi-Intent Decomposition and Fan-Out Pipeline

| Field    | Value                     |
|----------|---------------------------|
| Status   | Implemented               |
| Date     | 2026-03-19                |
| Authors  | Cedric                    |
| Replaces | —                         |

## Context

The PRD's core promise is "one voice command -> millions of agents." A user says "I want to move to Paris" and 47 agents across 9 domains activate simultaneously. The existing `Strategy::Swarm` with `GatherStrategy` (ADR-001) provides scatter-gather primitives, but there is no intent decomposition layer that maps a single natural language utterance into multiple parallel agent activations across different life domains.

The current pipeline handles one intent per utterance: STT produces a transcript, the TinyDancerRouter selects a single strategy, and one agent or one swarm scatter executes. Multi-domain life events require a fundamentally different approach where a single utterance fans out to dozens of agents across independent domains, some of which have cross-domain dependencies that must be resolved before execution can proceed.

## Decision

### Multi-Intent Decomposer

Introduce `MultiIntentDecomposer` in `rlmx-voice` that:

1. Takes a transcript + speaker context from STT
2. Classifies into 1+ of 12 life domains using domain embedding similarity
3. Extracts per-domain intents with action verbs and entity references
4. Produces a vector of `Intent` structs

```rust
pub struct Intent {
    pub domain: LifeDomain,       // 1 of 12 categories
    pub action: String,           // "cancel", "search", "compare", "analyze"
    pub entities: Vec<Entity>,    // extracted entities from utterance
    pub urgency: f32,             // 0-1, from voice features
    pub confidence: f32,          // decomposition confidence
}
```

The 12 life domains are: Visa/Immigration, Housing, Career, Education, Healthcare, Banking/Finance, Language, Logistics, Life Admin, Social/Community, Legal, and Entertainment/Culture.

Domain classification uses cosine similarity against pre-computed domain embedding centroids. A threshold of 0.45 activates a domain; domains below threshold are suppressed. The decomposer runs on the TieredEngine small model (0.5B) for latency, escalating to medium (3-8B) when confidence falls below 0.7.

### Fan-Out Pipeline

Complete flow:

```
User speaks
  -> VAD (voice activity detection)
  -> STT (speech-to-text)
  -> TinyDancerRouter (18-dim input)
  -> MultiIntentDecomposer
  -> Coordinator Agent
  -> ProcessFork x N
  -> Strategy::Swarm scatter (per-zone)
  -> Agents execute across zones
  -> SwarmEvent::AgentProgress streaming
  -> GatherStrategy
  -> Synthesized multimodal response
```

Each stage is a kernel syscall. The decomposer output feeds into the Coordinator as a batch of `Intent` structs rather than a single routed strategy. The Coordinator then issues one `ProcessFork` per intent, each carrying a time-bounded capability token scoped to the intent's domain.

### Coordinator Agent Enhancement

The Coordinator agent (Zone A-Mobile on phone) becomes the fan-out orchestrator:

1. Receives decomposed intents from `MultiIntentDecomposer`
2. Maps each intent to agent type(s) from the marketplace registry (ADR-014) or installed agents
3. Issues `ProcessFork` for each agent with time-bounded capability tokens (24h expiry, domain-scoped)
4. Manages progress tracking via `SwarmEvent::AgentProgress`
5. Applies `GatherStrategy` to collect results:
   - `GatherStrategy::All` -- wait for all domains (comprehensive analysis like "move to Paris")
   - `GatherStrategy::First` -- return first result (simple lookups)
   - `GatherStrategy::Quorum(n)` -- return when n domains complete (balanced speed/completeness)
   - `GatherStrategy::Streaming` -- stream results as they arrive with progressive rendering

### New GatherStrategy Variant

```rust
pub enum GatherStrategy {
    First,
    Quorum(usize),
    All,
    Streaming {              // NEW
        min_domains: usize,  // minimum domains before starting synthesis
        timeout_ms: u64,     // max wait for remaining domains
        progressive: bool,   // render partial results as they arrive
    },
}
```

`Streaming` differs from `All` in that the Coordinator begins synthesis after `min_domains` complete, then incrementally updates the response as remaining domains finish. If `timeout_ms` elapses, outstanding domains are marked incomplete and the user is notified which results are still pending. When `progressive` is true, the phone UI renders each domain's result card as soon as it arrives rather than waiting for the synthesis threshold.

### Progress Streaming

Each agent emits `SwarmEvent::AgentProgress` as it works:

```rust
SwarmEvent::AgentProgress {
    task_id: Uuid,
    agent_type: AgentType,
    domain: String,
    progress_pct: f32,        // 0.0 - 1.0
    status_text: String,      // "Scanning 12 arrondissements..."
    eta_ms: Option<u64>,      // estimated time remaining
}
```

The phone UI renders progress cards in real-time, one per domain. Cards transition through states: Queued -> Running (with progress bar) -> Complete (with summary) -> Actionable (with buttons). The WebSocket connection (ADR-008) carries these events with backpressure; if the client falls behind, intermediate progress updates are coalesced (latest wins per domain).

### Cross-Domain Dependency Resolution

For complex life events (move to Paris), agents have cross-domain dependencies:

- Schools depend on Housing (need neighborhood to find nearby schools)
- Enrollment depends on Visa (need legal status for eligibility)
- Banking depends on Career (need salary information for account requirements)
- Life Admin depends on Housing (need address for utility setup)

The Coordinator constructs a dependency DAG from domain relationship metadata. `qudag-dag` DAG consensus resolves execution order: dependent agents wait for prerequisite domains to complete, while non-dependent agents execute immediately in parallel. The DAG is validated for cycles at construction time; circular dependencies are broken by the Coordinator injecting placeholder values with a "pending confirmation" flag.

```
Visa ---------> Schools
                   |
Housing -------> Schools
     |---------> Life Admin
                   |
Career --------> Banking
                   |
Language ------> (no deps)
Logistics -----> (waits for all, executes last)
Healthcare ----> Visa
```

### "Move to Paris" Reference Implementation

47 agents across 9 domains:

| Domain | Agents | Count | Zone | Dependencies |
|--------|--------|-------|------|-------------|
| Visa | Visa Analyst, Document Prep | 2 | B, A-Mobile | None |
| Housing | Real Estate Scout, Neighborhood Analyst, Rent Calculator | 3 | B, B, A-Desktop | None |
| Career | Job Market, Resume Localizer, LinkedIn Optimizer, Salary Benchmarker | 4 | B, B, B, B | None |
| Schools | School Researcher, Enrollment Advisor | 2 | B, B | Housing, Visa |
| Healthcare | Insurance Comparator, Provider Finder, Prescription Transfer | 3 | B, B, A-Mobile | Visa |
| Banking | Account Migrator, Tax Treaty, Currency Optimizer, FATCA/FBAR | 4 | B, B, B, B | Career |
| Language | Level Assessor, Learning Planner | 2 | A-Mobile, B | None |
| Logistics | Moving Coordinator, Customs, Timeline Planner | 3 | B, B, cross-domain | All domains |
| Life Admin | Utility Setup, Address Management | 2 | B, A-Mobile | Housing |

Independent domains (Visa, Housing, Career, Language) start immediately. Dependent domains (Schools, Healthcare, Banking, Life Admin) start as soon as their prerequisites complete. Logistics starts last since it synthesizes across all domains to produce a unified timeline.

Total fan-out latency is bounded by the critical path through the dependency DAG, not by the sum of all agent execution times. For the "move to Paris" case, the critical path is: Career -> Banking (sequential) while Housing -> Schools runs in parallel. Expected wall-clock time: 15-45 seconds for the initial streaming results, 60-120 seconds for full completion across all 9 domains.

### Response Synthesis

After gather completes (or `min_domains` threshold is reached for `Streaming`):

1. Coordinator synthesizes a cross-domain summary, resolving conflicts between domain results
2. Ranks findings by actionability (immediate actions first, long-term planning last)
3. Generates an executive voice summary via TTS (~60 seconds, highlights only)
4. Renders visual cards per domain with action buttons on the phone UI
5. Provides "Start my [X]" buttons to begin executing individual plan items
6. Stores the full result set in the user's knowledge graph for later retrieval

### Budget and Rate Limiting

Each fan-out operation carries a compute budget derived from the user's capability token:

- Maximum agents per utterance: 100 (configurable per tier)
- Maximum concurrent cloud agents: 20 (Zone B capacity)
- Per-domain timeout: 30 seconds default, configurable per intent
- Total fan-out timeout: 120 seconds hard cap
- Cost tracking: each `ProcessFork` decrements the token's remaining budget

If the budget is exhausted mid-execution, remaining agents are cancelled gracefully and the Coordinator synthesizes from partial results.

## Consequences

### Positive

- Single utterance triggers comprehensive multi-domain analysis without manual orchestration
- Streaming progress provides immediate feedback (users see activity within 2-3 seconds, not silence for 2 minutes)
- DAG dependencies prevent agents from working with stale or missing cross-domain data
- `GatherStrategy::Streaming` enables progressive rendering, improving perceived latency
- Marketplace agents (ADR-014) slot directly into the fan-out pipeline via the registry mapping

### Negative

- Intent decomposition errors compound: wrong domain classification leads to wrong agents, wasting compute and returning irrelevant results
- Cross-domain dependency graph adds latency for dependent domains that must wait for prerequisites
- 47 concurrent agents require significant coordination overhead in the Coordinator and Zone B infrastructure
- The decomposer adds ~200ms latency to the voice pipeline before any agents start

### Risks

- **Decomposition accuracy**: Below 90% domain classification accuracy would frustrate users with irrelevant agent activations. Mitigation: confidence gating at 0.45 threshold, user confirmation for low-confidence domains.
- **Network failures**: Mid-execution failures leave partial results. Mitigation: graceful degradation with per-domain timeouts, partial synthesis, and "retry domain" buttons in the UI.
- **Cost management**: 47 cloud agents per query can be expensive. Mitigation: capability token budget caps, tiered inference (small model agents where possible), and user-configurable domain filtering ("only search Housing and Career").
- **Dependency deadlocks**: Circular or unresolvable dependencies stall the pipeline. Mitigation: cycle detection at DAG construction, placeholder injection for circular refs, per-domain timeouts as a backstop.

## References

- ADR-001: Distributed Swarm Architecture (`Strategy::Swarm`, `GatherStrategy`)
- ADR-005: Capability-Secured Agents (time-bounded tokens, domain-scoped permissions)
- ADR-008: WebSocket Real-Time Events (`SwarmEvent` streaming, backpressure)
- ADR-011: Sandbox Orchestration (agent resource envelopes, fleet deployment)
- ADR-012: Voice-First Pipeline (STT -> Intent flow, VAD)
- ADR-013: Phone Command Center (mobile Coordinator, progress UI)
- ADR-014: Agent Marketplace (registry mapping for intent -> agent resolution)
- PRD Section 5: Voice -> Millions of Agents

## Implementation Notes

Implemented via MultiIntentDecomposer in `crates/rlmx-voice/src/intent.rs`. Single utterance decomposes into multiple Intent structs across 12 life domains using keyword-based domain classification with confidence thresholds. Coordinator dispatches via ProcessFork with time-bounded capability tokens. Fan-out pipeline: VAD -> STT -> TinyDancerRouter -> MultiIntentDecomposer -> Coordinator -> ProcessFork x N -> Strategy::Swarm scatter -> GatherStrategy.
