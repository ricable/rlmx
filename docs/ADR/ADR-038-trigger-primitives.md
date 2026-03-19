# ADR-038: Trigger Primitives

Status: Implemented

## Context

RLMX agents need to react to external events — HTTP webhooks, cron schedules, domain events, and channel messages. Without a unified trigger system, each integration point implements its own ad-hoc routing, making the system fragile and hard to extend.

AgentOS defines Worker/Function/Trigger primitives that bind events to functions. This ADR adapts the trigger registry concept to RLMX, focusing on the binding and matching layer while deferring actual function execution to the existing process model (ADR-007).

## Decision

Implement a `TriggerRegistry` in `rlmx-kernel` and a corresponding `@aix/triggers` TypeScript package that provide:

### Trigger Types

| Type | Source | Match Criteria |
|------|--------|---------------|
| **Http** | Incoming webhook | Path pattern + HTTP method |
| **Schedule** | Cron timer | Cron expression |
| **Event** | Domain event bus | Event type string |
| **Channel** | Channel adapter (ADR-039) | Adapter name + content filter |

### Trigger Bindings

A `TriggerBinding` connects a trigger source to a target function with optional input transformation:

```
TriggerBinding:
  id:              UUID
  trigger_type:    TriggerType
  target_function: String    (function name or process template)
  transform:       Option<String>  (jq-like expression for input mapping)
  enabled:         bool
  max_depth:       u8  (default 5, cycle prevention)
```

### Cycle Prevention

Triggers can cause cascading function invocations. The registry tracks active recursion depth per function and refuses to fire if `depth >= max_depth`. This prevents infinite trigger loops.

### Matching

- `match_event(event_type)` — returns all enabled Event bindings for the given type
- `match_http(path, method)` — returns all enabled Http bindings matching path and method
- `match_channel(adapter, content)` — returns all enabled Channel bindings where adapter matches and content contains the filter string

## Consequences

- Unified event-to-function routing across all input sources
- Cycle prevention guarantees termination of cascading triggers
- Transform expressions enable input normalization without custom code
- TypeScript mirror enables browser/Node trigger evaluation
- Schedule triggers define intent only; actual cron execution is deferred to the scheduler (ADR-007)
