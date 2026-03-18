# DDD-006: Research & Evolution Bounded Context

## Overview

The Research & Evolution context implements autonomous discovery: agents
generate hypotheses, run experiments in COW-branched isolation, discover
beneficial mutations, and cross-pollinate successful patterns across the
swarm. When local resources stall, the system escalates to cloud compute.

**Crate**: `crates/rlmx-agents/` (researcher.rs, experimenter.rs, mutation.rs, experiment.rs)

## Aggregate Root: ResearchObjective

```rust
pub struct ResearchObjective {
    pub id: Uuid,
    pub description: String,
    pub researchers: Vec<AgentId>,
    pub experiments: Vec<Experiment>,
    pub best_genome: Option<Genome>,
    pub generation: Generation,
    pub status: ResearchStatus,      // Active, Stalled, Completed, Escalated
    pub started_at: DateTime<Utc>,
    pub stall_threshold: Duration,   // triggers cloud escalation
}
```

`ResearchObjective` is the aggregate root because it owns the consistency
boundary for the research lifecycle: which experiments are running, what the
current best genome is, and when to escalate.

## Entities

### Researcher

An agent (type `AgentType::Researcher`) that manages the research pipeline:

```rust
pub struct ResearcherState {
    pub agent_id: AgentId,
    pub objective: Uuid,
    pub hypotheses: Vec<Hypothesis>,
    pub active_experiments: Vec<Uuid>,
    pub prior_patterns: Vec<PatternRef>,  // from SONA PatternBank via VecSearch
}
```

### Experimenter

An agent (type `AgentType::Experimenter`) that executes individual experiments:

```rust
pub struct ExperimenterState {
    pub agent_id: AgentId,
    pub experiment: Experiment,
    pub branch_id: Uuid,       // COW branch from BranchManager
    pub mutations_applied: Vec<MutationDelta>,
}
```

### MutationStrategy

A serializable genome that encodes system configuration:

```rust
pub struct MutationStrategy {
    pub id: Uuid,
    pub genome: Genome,
    pub parent_id: Option<Uuid>,       // lineage tracking
    pub generation: Generation,
    pub fitness: Option<FitnessScore>,
    pub created_at: DateTime<Utc>,
}
```

### Experiment

```rust
pub struct Experiment {
    pub id: Uuid,
    pub hypothesis: Hypothesis,
    pub strategy: MutationStrategy,
    pub branch_id: Uuid,
    pub evidence: Vec<EvidenceItem>,
    pub fitness: Option<FitnessScore>,
    pub generation: Generation,
    pub status: ExperimentStatus,  // Pending, Running, Completed, Failed
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
```

### CrossPollinator

A service that transfers successful mutations between research objectives
and across swarm nodes:

```rust
pub struct CrossPollinator {
    pub transfer_log: Vec<CrossPollinationRecord>,
}

pub struct CrossPollinationRecord {
    pub source_objective: Uuid,
    pub target_objective: Uuid,
    pub genome_fragment: Genome,
    pub fitness_at_source: FitnessScore,
    pub transferred_at: DateTime<Utc>,
}
```

### FitnessEvaluator

Computes multi-objective fitness from experiment results:

```rust
pub struct FitnessEvaluator {
    pub accuracy_weight: f64,   // default: 0.5
    pub latency_weight: f64,    // default: 0.3
    pub cost_weight: f64,       // default: 0.2
}

impl FitnessEvaluator {
    pub fn evaluate(&self, accuracy: f64, latency_ms: f64, cost: f64) -> FitnessScore;
}
```

## Value Objects

| Value Object | Definition |
|-------------|------------|
| `Hypothesis` | `{ id: Uuid, statement: String, predicted_improvement: f64, confidence: f64 }` |
| `Genome` | `{ feature_weights: Vec<f64>, routing_thresholds: Vec<f64>, prompt_templates: Vec<String>, attention_config: String }` |
| `FitnessScore` | `{ accuracy: f64, latency_ms: f64, cost: f64, combined: f64 }`. Compared by `combined`. |
| `Generation` | Newtype `u32`. Monotonically increasing per research objective. |
| `MutationDelta` | `{ field: String, old_value: Value, new_value: Value, mutation_type: MutationType }` |
| `MutationType` | Enum: `PointMutation`, `Crossover`, `Insertion`, `Deletion` |
| `EvidenceItem` | `{ metric: String, value: f64, measured_at: DateTime<Utc> }` |
| `ExperimentStatus` | Enum: `Pending`, `Running`, `Completed`, `Failed` |
| `ResearchStatus` | Enum: `Active`, `Stalled`, `Completed`, `Escalated` |

## Domain Events

| Event | Trigger | Data |
|-------|---------|------|
| `ResearchStarted` | New `ResearchObjective` created | `{ objective_id, description, researcher_ids }` |
| `HypothesisGenerated` | Researcher produces hypothesis | `{ objective_id, hypothesis_id, statement, confidence }` |
| `ExperimentStarted` | Experimenter begins with COW branch | `{ experiment_id, hypothesis_id, branch_id, genome }` |
| `ExperimentCompleted` | Experimenter finishes | `{ experiment_id, fitness, evidence, generation }` |
| `MutationDiscovered` | Fitness above threshold | `{ strategy_id, genome, fitness, generation }` |
| `CrossPollinated` | Successful genome transferred | `{ source_obj, target_obj, genome_fragment, fitness }` |
| `CloudEscalated` | Research stalled beyond threshold | `{ objective_id, stall_duration, reason }` |
| `ResearchSynthesized` | Objective completed | `{ objective_id, best_genome, total_experiments, generations }` |

## Domain Services

### `research()` -- 10-step auto-research pipeline

```
Step 1: Define objective
    |
Step 2: Search prior patterns (VecSearch -> SONA PatternBank)
    |
Step 3: Generate hypotheses (Researcher agents, max 3 concurrent)
    |
Step 4: Design experiments (one per hypothesis)
    |
Step 5: Create COW branches (BranchManager::create_branch())
    |
Step 6: Execute experiments (Experimenter agents, max 8 total)
    |
Step 7: Evaluate fitness (FitnessEvaluator)
    |
Step 8: Select survivors (top-k by FitnessScore.combined)
    |
Step 9: Mutate + crossover (produce next generation genome)
    |
Step 10: Cross-pollinate or escalate
    |
    +-- If improvement found: store in SONA PatternBank, goto Step 3
    +-- If stalled > threshold: escalate to cloud
    +-- If fitness target met: synthesize and complete
```

### `mutate(genome: &Genome) -> Genome`

Applies random mutations to a genome:
- **Point mutation**: Perturb a single `feature_weight` by gaussian noise.
- **Crossover**: Combine two parent genomes at a random split point.
- **Insertion**: Add a new prompt template variant.
- **Deletion**: Remove the lowest-contributing feature weight.

### `cross_pollinate(source_obj, target_obj, genome_fragment)`

Transfers a genome fragment from one research objective to another:
1. Extract the most-improved fields from the source genome.
2. Validate that the fragment is compatible with the target's search space.
3. Inject into the target's next generation.
4. Record in `CrossPollinationRecord`.

### `escalate_to_cloud(objective)`

When a research objective stalls (no fitness improvement for `stall_threshold`
duration):
1. Serialize the current best genome and all experiment evidence.
2. Route to a `Cloud` zone node with GPU capability.
3. Run larger-scale experiments with higher-tier models.
4. Results flow back as `ExperimentCompleted` events.

## Invariants

1. **Max 3 concurrent Researchers**: Enforced by `AgentRegistry` limits.

2. **Max 8 Experimenters total**: Across all research objectives. Enforced at
   spawn time.

3. **COW branch isolation**: Each experiment runs on its own
   `BranchManager::create_branch()` fork. Mutations in one experiment cannot
   affect another or the main container state.

4. **Cloud escalation only on stall**: The system only escalates to cloud
   compute when local research has stalled beyond `stall_threshold` (default:
   5 minutes of no fitness improvement). This prevents unnecessary cloud costs.

5. **Fitness monotonicity per objective**: The `best_genome` on a
   `ResearchObjective` is only updated when a new genome has strictly higher
   `FitnessScore.combined` than the current best.

6. **Generation ordering**: Experiments within a generation must all complete
   before the next generation begins. Partial results do not advance the
   generation counter.

## Integration with Other Contexts

### Kernel Syscall (via ProcessFork + StateMutate)

- Researchers and Experimenters are kernel processes with scoped capability
  tokens. Experimenters need `StateMutate` permission to record their findings
  in the proof chain.

### Observation & Health (via SONA PatternBank)

- Successful mutations (fitness above threshold) are stored as patterns via
  `Sona::record_pattern()`.
- New research starts by querying `Sona::find_similar_patterns()` for prior
  knowledge about similar objectives.

### Container & Storage (via BranchManager)

- Each experiment creates a COW branch of the current RVF container.
- Successful experiments merge their branch back via `BranchManager::merge()`.
- Failed experiments' branches are discarded.

### Swarm Coordination (via cloud escalation)

- When research stalls, the swarm routes the work to Cloud zone nodes with
  GPU resources.
- Uses `SwarmOrchestrator::rebalance()` to find available cloud capacity.

## File Map (planned)

| File | Types |
|------|-------|
| `crates/rlmx-agents/src/researcher.rs` | `ResearcherState`, hypothesis generation |
| `crates/rlmx-agents/src/experimenter.rs` | `ExperimenterState`, experiment execution |
| `crates/rlmx-agents/src/mutation.rs` | `MutationStrategy`, `Genome`, `MutationDelta`, `mutate()` |
| `crates/rlmx-agents/src/experiment.rs` | `Experiment`, `ResearchObjective`, `FitnessEvaluator` |
| `crates/rlmx-agents/src/crossover.rs` | `CrossPollinator`, genome transfer |
| `crates/rlmx-agents/src/escalation.rs` | Cloud escalation logic |
