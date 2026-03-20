# ADR-037: Approval Tiers

Status: Implemented

## Context

RLMX agents autonomously execute syscalls, spawn processes, mutate state, and incur costs. Without human-in-the-loop gates, a misbehaving or compromised agent can drain budgets, corrupt data, or trigger irreversible operations before an operator notices. The existing capability token system (ADR-005) controls *who* can call *what*, but not *whether a human should review* the action first.

AgentOS defines a four-tier approval model that balances autonomy (most operations proceed instantly) with safety (dangerous or expensive operations require human confirmation). This ADR adapts that model to RLMX's syscall and domain-event architecture.

## Decision

Implement an `ApprovalGate` in `rlmx-kernel` that classifies every operation into one of four tiers and blocks Confirm/Escalate operations until a human decides.

### Approval Tiers

| Tier | Behavior | Latency | Example |
|------|----------|---------|---------|
| **Auto** | Proceed immediately, no notification | 0 ms | VecSearch, GraphQuery, AttentionSelect |
| **Notify** | Proceed immediately, notify human after | 0 ms + async notify | VecInsert, VecDelete, ArtifactWrite, ProcessFork |
| **Confirm** | Block until human approves or timeout | seconds–minutes | StateMutate, MeshSync, FederationContribute |
| **Escalate** | Block, require admin-level approval | minutes–hours | BillingManager ops, security-critical mutations |

### Cost-Based Escalation

Any operation whose `cost_estimate` exceeds the policy's `cost_threshold` is automatically escalated one tier (e.g., a Notify operation with cost > threshold becomes Confirm).

### Timeout Auto-Deny

Pending approval requests that exceed the configured timeout (default 5 minutes) are automatically denied. This prevents indefinite blocking of agent pipelines.

### Domain Events

- `ApprovalRequested` — emitted when a Confirm/Escalate request is created
- `ApprovalDecided` — emitted when a human approves or denies (or timeout auto-denies)

### Integration Points

- **MCP**: `approval.list`, `approval.decide` tools
- **WebSocket**: `ApprovalRequired` swarm event pushed to dashboard
- **Mobile**: Push notification for Escalate-tier requests
- **CLI**: Interactive prompt for Confirm-tier when running locally

## Consequences

- Agents calling Confirm/Escalate operations will block until decided
- Auto and Notify operations are zero-latency (no regression)
- Timeout auto-deny prevents zombie approval requests
- Cost thresholds provide automatic safety escalation for expensive operations
- Policies are configurable per-deployment via `ApprovalGate::add_policy`
