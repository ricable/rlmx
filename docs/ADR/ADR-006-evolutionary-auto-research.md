# ADR-006: Evolutionary Auto-Research via Agent Swarm

## Status
Proposed

## Date
2026-03-18

## Context
The RLMX distributed swarm (25 nodes) needs an automated mechanism to discover
improvements to model training, prompt templates, routing thresholds, and feature
weights without human-in-the-loop tuning. Manual hyperparameter search does not
scale across a heterogeneous cluster (Mac Metal, NUC-GPU CUDA, RPi5 edge, browser
WASM nodes). The existing `rlmx-cognitive` crate already provides SONA pattern
banking and `DagOptimizer` strategy tracking, but these operate reactively on
individual queries rather than proactively exploring the search space.

Drew Karpathy's evolutionary auto-research paradigm -- generate N hypotheses,
run short experiments in parallel, cross-pollinate winners, iterate -- maps
naturally onto a multi-agent swarm where each Experimenter runs on a separate
compute node with COW-branched state isolation.

## Decision
Implement an evolutionary auto-research loop as a first-class swarm workflow
coordinated by four agent roles:

### Agent Roles

1. **Coordinator** (`crates/rlmx-agents/src/coordinator.rs`): Owns the research
   lifecycle. Receives a research goal (e.g., "reduce val_bpb on wikitext by 5%"),
   spawns a Researcher, collects final results, and integrates the winning mutation
   into the canonical state via `StateMutate` syscall.

2. **Researcher** (`crates/rlmx-agents/src/researcher.rs`): Searches prior
   patterns via `VecSearch` syscall against SONA pattern bank. Generates N
   hypothesis `MutationStrategy` genomes. After experiments complete, synthesizes
   results and selects the winner based on val_bpb ranking.

3. **Experimenter** (`crates/rlmx-agents/src/experimenter.rs`): Receives a
   `MutationStrategy` genome and a COW-branched `RvfContainer` (via
   `BranchManager::create_branch()`). Mutates `train.py` parameters using the
   local ruvltra model for code generation. Requests a Trainer spawn on the
   appropriate compute backend.

4. **Reviewer** (`crates/rlmx-agents/src/reviewer.rs`): Validates the winning
   mutation against safety constraints (`SafetyEngine::check_bounds()`), checks
   that val_bpb improvement exceeds the significance threshold (default 1%),
   and verifies the witness chain integrity of the experiment's `RvfContainer`.

### MutationStrategy Genome

Defined in `crates/rlmx-agents/src/mutation.rs`:

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct MutationStrategy {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub generation: u32,
    pub feature_weights: HashMap<String, f64>,
    pub routing_thresholds: RoutingThresholds,
    pub prompt_templates: Vec<PromptTemplate>,
    pub training_config: TrainingConfig,
    pub fitness: Option<f64>,  // val_bpb (lower is better)
}
```

### Experiment Flow

1. Coordinator sends research goal to Researcher.
2. Researcher queries `VecSearch` for related SONA patterns (top-k=20).
3. Researcher generates N `MutationStrategy` genomes (default N=6).
4. Coordinator spawns N Experimenters, each receiving one genome + COW branch.
5. Each Experimenter mutates training code, spawns a Trainer on its node:
   - Mac nodes: MLX backend via `ProcessFork` with `metal` capability.
   - NUC-GPU nodes: CUDA backend via `ProcessFork` with `cuda` capability.
   - RPi5 nodes: excluded from training (inference-only).
6. Training runs for a bounded duration (default 5 minutes).
7. Experimenters report `val_bpb` to Coordinator via `ProcessSend`.
8. Cross-pollination: top-2 genomes are recombined via weighted averaging of
   `feature_weights` and tournament selection of `prompt_templates`.
9. If no genome improves over baseline after 3 generations, Coordinator escalates
   to cloud (Sonnet/Opus) for hypothesis refinement per ADR-026 Tier 3.
10. Researcher synthesizes final report; Reviewer validates.
11. Coordinator integrates winner: stores as SONA pattern, updates `DagOptimizer`
    success rates, commits via `StateMutate` with witness chain.

### Gossip Cross-Pollination

Experimenters on different nodes exchange intermediate fitness scores via the
swarm gossip protocol (`crates/rlmx-swarm/src/gossip.rs`). This enables early
termination of underperforming experiments and reallocation of compute to
promising directions.

## Consequences

### Positive
- Automated discovery of improvements without manual tuning cycles.
- Full utilization of heterogeneous compute (Mac Metal for fast MLX training,
  CUDA NUCs for larger experiments).
- COW branching via `rlmx-rvf` provides zero-cost state isolation -- failed
  experiments never pollute canonical state.
- Successful mutations persist as SONA patterns, building institutional memory.
- Bounded experiment duration (5 min) prevents runaway resource consumption.

### Negative
- Increased complexity in agent coordination -- 4 new agent roles.
- Training workloads may saturate smaller nodes (RPi5 excluded, but Mac/NUC
  could be busy).
- Cross-pollination via gossip introduces eventual consistency in fitness
  rankings.

### Risks
- Degenerate mutations could produce valid but useless training configs that
  waste compute. Mitigated by Reviewer validation and `SafetyEngine` bounds.
- Cloud escalation (Tier 3) incurs API costs. Mitigated by 3-generation local
  threshold before escalation.
- Experiment isolation depends on `BranchManager` correctness -- a bug could
  allow mutation of canonical state. Mitigated by witness chain verification
  in Reviewer.

## Alternatives Considered

1. **Manual hyperparameter tuning**: Does not scale to 25-node swarm. Requires
   human expertise for each adjustment. Rejected.

2. **Centralized Bayesian optimization (e.g., Optuna)**: Single point of failure,
   does not leverage distributed compute naturally, requires a centralized
   parameter server. Would need a new dependency. Rejected in favor of the
   agent-native approach.

3. **No evolution (static configuration)**: Simplest option but leaves
   performance improvements on the table. The swarm hardware is already
   available -- not using it for self-improvement is wasteful. Rejected.

## References
- Karpathy, A. "Evolutionary Auto-Research" (2025) -- inspiration for the
  generate-experiment-select loop.
- `crates/rlmx-cognitive/src/sona.rs` -- SONA pattern bank for storing
  successful mutations.
- `crates/rlmx-rvf/src/branch.rs` -- `BranchManager` for COW state isolation.
- `crates/rlmx-kernel/src/syscall.rs` -- `VecSearch`, `ProcessFork`,
  `ProcessSend`, `StateMutate` syscalls.
- ADR-026 (3-Tier Model Routing) -- cloud escalation path.
