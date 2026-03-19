# ADR-032: Per-Agent Budget Ledger

| Field       | Value                                         |
|-------------|-----------------------------------------------|
| Status      | Implemented                                   |
| Date        | 2026-03-19                                    |
| Deciders    | RLMX Core Team                                |
| Supersedes  | —                                             |
| Related     | ADR-025 (Subscription Billing Tiers), ADR-029 |

## Context

RLMX supports spawning many concurrent agents per user, each consuming LLM inference
tokens across different models and providers. Without per-agent cost tracking, users
have no visibility into which agents are driving spend, and the system cannot enforce
fine-grained budget controls.

## Decision

Introduce a **Per-Agent Budget Ledger** that:

1. **Tracks cost per agent** — every LLM call records a `BudgetEntry` containing
   agent ID, model, provider, token counts (in/out), cost in microcents, and timestamp.

2. **Enforces soft and hard limits** via `BudgetPolicy`:
   - **Soft limit**: when crossed, the `BudgetDecision` includes a warning string but
     the call is still allowed.
   - **Hard limit**: when crossed, the call is denied (`allowed: false`).
   - **Per-call max tokens**: optional cap on a single inference call's token count.

3. **Uses CAS (Compare-and-Swap) versioning** on the ledger to prevent concurrent
   writes from producing inconsistent totals. Each `record()` call accepts an optional
   `expected_version`; if the current version does not match, the write is rejected
   with an error.

4. **Integrates with tier enforcement** by adding a `BudgetLimit(u64)` variant to
   `TierCaveat`, allowing subscription tiers to set default budget caps per agent.

5. **Reports** per-agent spend breakdowns by model and provider.

## Implementation

### Rust (`rlmx-billing/src/budget.rs`)

- `BudgetEntry` — immutable record of a single LLM call's cost.
- `BudgetPolicy` — soft limit, hard limit, optional per-call token cap.
- `BudgetDecision` — result of `check()`: allowed flag, optional warning, remaining microcents.
- `BudgetLedger` — in-memory HashMap-based ledger with CAS versioning.

### TypeScript (`@aix/billing/src/budget.ts`)

- Mirror types and `BudgetLedger` class using `Map<string, ...>`.
- Integrates with `@aix/shared` for ID generation.

### Capability Enforcement

- New `TierCaveat::BudgetLimit(u64)` variant in Rust.
- New `{ type: 'BudgetLimit'; limit: number }` variant in TypeScript.

## Consequences

- Agents can be individually budget-constrained without affecting other agents.
- CAS versioning prevents lost updates in concurrent recording scenarios.
- Soft limits provide early warning; hard limits provide hard stops.
- Per-model/provider reporting enables cost optimization decisions.
- No policy = unlimited: backward compatible with existing agents.
