# ADR-036: Dynamic Function Evolution

**Status:** Implemented
**Date:** 2026-03-19
**Deciders:** Core team
**Supersedes:** None

## Context

RLMX agents need the ability to evolve their own functions at runtime — creating,
testing, scoring, and promoting (or killing) functions through a structured lifecycle.
AgentOS demonstrated this pattern with a node:vm sandbox, score-weighted feedback,
and DAG-based version tracking. We adapt these ideas for the RLMX kernel with
stronger isolation (WASM sandbox via `SandboxManager`) and tighter integration with
the existing artifact DAG (ADR-030), budget ledger (ADR-032), and approval gates
(ADR-037).

## Decision

### Lifecycle

Every evolved function progresses through a strict state machine:

```
Draft -> Staging -> Production -> Deprecated
  |        |           |             |
  +--------+-----------+-------------+--> Killed
```

Valid transitions:
- Draft -> Staging (passes dual-gate validation)
- Staging -> Production (sustained score above threshold)
- Production -> Deprecated (superseded by newer version)
- Any -> Killed (fatal failures or manual intervention)

Skip transitions (e.g., Draft -> Production) are rejected.

### Dual-Gate Security

Before a function moves from Draft to Staging it must pass two independent gates:

1. **Static scan** — AST-level checks for forbidden APIs, infinite loops,
   resource abuse. Runs outside sandbox.
2. **Dynamic sandbox test** — the function is executed inside the WASM
   `SandboxManager` with a curated test vector. Must complete within timeout
   and produce the expected output.

Both gates must pass; failure on either blocks the transition and emits a
`FunctionEvolved` domain event with error details.

### Scoring Formula

Each execution of a function produces a `ScoreResult`:

```
overall = correctness * 0.50
        + safety      * 0.25
        + latency_score * 0.15
        + cost_score    * 0.10
```

Where:
- `correctness` — 0.0 or 1.0 (exact match) or 0.0-1.0 (LLM judge / semantic similarity)
- `safety` — 0.0-1.0 from sandbox telemetry (memory, syscalls, time)
- `latency_score` — `clamp(1.0 - actual_ms / timeout_ms, 0.0, 1.0)`
- `cost_score` — `clamp(1.0 - cost / budget_limit, 0.0, 1.0)`

### Scorer Types

| Scorer | Output | Use case |
|--------|--------|----------|
| `exact_match` | 0 or 1 | Deterministic outputs |
| `semantic_similarity` | 0.0-1.0 | Jaccard word-set overlap |
| `llm_judge` | 0.0-1.0 | Subjective quality (future) |
| `custom` | 0.0-1.0 | Domain-specific (future) |

### Auto-Scoring Modes

| Mode | Behavior |
|------|----------|
| `Auto` | Score every invocation |
| `Sampled` | Score 10% of invocations (random) |
| `Manual` | Only score when explicitly requested |
| `Off` | No scoring |

### Feedback Loop

The `FeedbackEngine` reviews the last 5 evaluation results and decides:

- **KILL** — 3 or more of the last 5 have `correctness < 0.5`.
  Emits `FunctionKilled` domain event. Status forced to `Killed`.
- **IMPROVE** — average `overall` score across last 5 is below 0.5.
  Triggers a re-evolution attempt (max 3 recursive iterations).
- **KEEP** — function is performing acceptably.

An auto-review cron runs every 6 hours for all `Staging` and `Production`
functions.

### DAG Branching

Functions form a directed acyclic graph:

- **fork(parent_id)** — create a new Draft branched from any existing version.
- **lineage(id)** — trace ancestry back to root (max depth 100, cycle-safe).
- **leaves(name)** — frontier nodes (no children, excludes Killed).
- **children(id)** — direct descendants.

The DAG is stored separately from the artifact DAG (ADR-030) because function
evolution has domain-specific invariants (status filtering, lineage limits).

### SONA Integration

When a function is promoted to Production, its embedding is registered with the
SONA adapter so that future agent queries can discover and invoke it via
semantic routing.

When a function is Killed, its SONA registration is removed.

### Domain Events

Three new `DomainEvent` variants (already added to `events.rs` and `@aix/shared`):

| Event | Emitted when |
|-------|-------------|
| `FunctionEvolved` | Function created or transitions status |
| `FunctionScored` | A score result is recorded |
| `FunctionKilled` | Function status set to Killed |

## Consequences

### Positive
- Agents can self-improve by evolving and testing new function implementations
- Dual-gate security prevents unsafe code from reaching production
- Score-driven feedback loop automates quality management
- DAG branching enables safe experimentation without losing known-good versions

### Negative
- Additional complexity in the runtime (mitigated by clear state machine)
- Scoring adds latency to each invocation in Auto mode (mitigated by Sampled/Off modes)
- Max 3 recursive improvement iterations may not converge (mitigated by KILL threshold)

### Neutral
- Function code is stored as strings; compilation/interpretation is delegated to SandboxManager
- The DAG is in-memory; persistence is handled by the artifact store if needed

## Implementation

- Rust crate: `rlmx-evolve` (`crates/rlmx-evolve/`)
- TypeScript package: `@aix/evolve` (`packages/evolve/`)
- Both implement identical logic: lifecycle state machine, scoring formula,
  feedback engine, DAG operations.
