# ADR-010: MCP Tool Expansion from 15 to 26 Tools

## Status
Proposed

## Date
2026-03-18

## Context
The RLMX MCP server currently exposes 15 tools (12 core syscall-mapped tools +
3 edge inference tools) via JSON-RPC 2.0 in `crates/rlmx-mcp/src/tools.rs`.
The 25-node distributed swarm introduces new operational domains -- swarm
management, agent lifecycle, evolutionary research, and forecasting -- that
need programmatic access through the same MCP interface used by LLM agents
and the Svelte dashboard.

Adding these as MCP tools (rather than separate APIs) ensures:
- Uniform access control via the existing 6-role RBAC model.
- LLM agents can orchestrate swarm operations using the same tool-calling
  interface they use for kernel syscalls.
- The dashboard uses a single JSON-RPC endpoint for all operations.

## Decision
Expand the MCP tool set by 11 new tools across three domains: swarm management,
agent lifecycle, and research/experimentation. All tools are defined in the
existing `crates/rlmx-mcp/src/tools.rs` file and dispatched through the
existing `handle_tool_call()` function in `crates/rlmx-mcp/src/server.rs`.

### New Tool Definitions

#### Swarm Management (2 tools)

| Tool | Min Role | Parameters | Returns |
|------|----------|------------|---------|
| `rlmx_swarm_status` | Viewer | `{}` | `{ nodes: [...], topology, health_summary }` |
| `rlmx_swarm_topology` | Viewer | `{ format?: "json" \| "dot" }` | Topology graph as JSON or Graphviz DOT |

#### Agent Lifecycle (3 tools)

| Tool | Min Role | Parameters | Returns |
|------|----------|------------|---------|
| `rlmx_agent_spawn` | Engineer | `{ agent_type, node_id?, config? }` | `{ agent_id, node_id, status }` |
| `rlmx_agent_list` | Operator | `{ node_id?, agent_type?, status? }` | `{ agents: [...] }` |
| `rlmx_agent_terminate` | Admin | `{ agent_id, reason? }` | `{ success, terminated_at }` |

#### Research and Experimentation (6 tools)

| Tool | Min Role | Parameters | Returns |
|------|----------|------------|---------|
| `rlmx_research_start` | Engineer | `{ goal, max_generations?, budget_minutes? }` | `{ research_id, status }` |
| `rlmx_research_status` | Operator | `{ research_id }` | `{ generation, best_fitness, active_experiments }` |
| `rlmx_experiment_list` | Viewer | `{ research_id?, status?, limit? }` | `{ experiments: [...] }` |
| `rlmx_mutation_history` | Viewer | `{ research_id?, generation?, limit? }` | `{ mutations: [...], lineage_graph }` |
| `rlmx_forecast` | Operator | `{ metric, horizon_hours, model? }` | `{ predictions: [...], confidence }` |
| `rlmx_train` | Engineer | `{ config, node_id?, backend? }` | `{ training_id, node_id, backend }` |

### Tool Registration

Each tool is registered in the `TOOL_DEFINITIONS` array with its JSON Schema
input schema and minimum RBAC role:

```rust
// crates/rlmx-mcp/src/tools.rs
ToolDefinition {
    name: "rlmx_agent_spawn",
    description: "Spawn a new agent on a swarm node",
    input_schema: json!({
        "type": "object",
        "properties": {
            "agent_type": { "type": "string", "enum": AGENT_TYPES },
            "node_id": { "type": "string", "format": "uuid" },
            "config": { "type": "object" }
        },
        "required": ["agent_type"]
    }),
    min_role: Role::Engineer,
}
```

### RBAC Enforcement

The existing RBAC model in `server.rs` is unchanged. Role hierarchy:
`Viewer < Operator < Engineer < Admin < Auditor < System`. The
`check_permission()` function in `server.rs` compares the caller's role
(resolved from `token_roles` in `McpConfig`) against the tool's `min_role`.

Critical constraint preserved: **clients cannot self-escalate**. The `_role`
parameter in `initialize` rejects `admin` and `system` values. Only
server-side `token_roles` configuration can assign privileged roles.

`rlmx_agent_terminate` requires Admin because terminating agents can disrupt
active experiments. `rlmx_train` requires Engineer because it allocates GPU
resources on specific nodes.

### Dispatch Architecture

The `handle_tool_call()` match statement in `server.rs` is extended with new
arms. Each new tool delegates to a handler module:

```rust
// crates/rlmx-mcp/src/server.rs (in handle_tool_call)
"rlmx_swarm_status" => handlers::swarm::status(ctx, params).await,
"rlmx_swarm_topology" => handlers::swarm::topology(ctx, params).await,
"rlmx_agent_spawn" => handlers::agent::spawn(ctx, params).await,
"rlmx_agent_list" => handlers::agent::list(ctx, params).await,
"rlmx_agent_terminate" => handlers::agent::terminate(ctx, params).await,
"rlmx_research_start" => handlers::research::start(ctx, params).await,
"rlmx_research_status" => handlers::research::status(ctx, params).await,
"rlmx_experiment_list" => handlers::research::experiments(ctx, params).await,
"rlmx_mutation_history" => handlers::research::mutations(ctx, params).await,
"rlmx_forecast" => handlers::research::forecast(ctx, params).await,
"rlmx_train" => handlers::research::train(ctx, params).await,
```

Handler modules live in `crates/rlmx-mcp/src/handlers/` with files:
`swarm.rs`, `agent.rs`, `research.rs`.

### CLI Subcommands

The CLI (`crates/rlmx-cli/src/main.rs`) gains new subcommands that call these
tools via the MCP client:

```
rlmx swarm start [--topology hierarchical] [--max-nodes 25]
rlmx swarm status
rlmx swarm topology [--format dot]
rlmx swarm chaos [--type network-partition] [--duration 30s]
rlmx agent spawn <type> [--node <id>] [--config <json>]
rlmx agent list [--type <type>] [--status <status>]
rlmx agent kill <agent-id> [--reason <reason>]
rlmx research start <goal> [--generations 10] [--budget 60]
rlmx research status <research-id>
rlmx research list [--status active]
```

### Initialization Handshake

The existing MCP initialization handshake (ADR: `server.rs` enforces
`initialize` before `tools/list` or `tools/call`) remains mandatory. The
`tools/list` response now returns 26 tool definitions. Clients that cache
tool lists must refresh after server upgrade.

### Backward Compatibility

All 15 existing tools are unchanged in name, schema, and behavior. The
expansion is purely additive. Existing clients that only use the original
15 tools are unaffected.

## Consequences

### Positive
- Single unified interface for all RLMX operations -- kernel, edge, swarm,
  agents, and research.
- LLM agents can autonomously manage the swarm via tool calls without custom
  integration code.
- Dashboard uses one JSON-RPC endpoint for everything, simplifying frontend
  architecture.
- Consistent RBAC across all operations -- no separate auth for swarm vs kernel.

### Negative
- `tools.rs` grows significantly. May need to split into `tools/core.rs`,
  `tools/edge.rs`, `tools/swarm.rs` modules if it exceeds 500 lines
  (per CLAUDE.md convention).
- 26 tools may overwhelm LLM context windows when returned by `tools/list`.
  Mitigated by clear naming conventions (`rlmx_` prefix + domain) and concise
  descriptions.
- More tools means more attack surface for RBAC bypass attempts. Mitigated
  by the existing `check_permission()` gate that applies uniformly.

### Risks
- `rlmx_agent_terminate` with Admin role could be used to disrupt the swarm
  if an Admin token is compromised. Mitigated by audit logging via
  `StateMutate` witness chain for all destructive operations.
- `rlmx_train` could monopolize GPU resources. Mitigated by the scheduler's
  resource tracking and per-node concurrent training limits (default 1).
- Research tools depend on the evolutionary auto-research system (ADR-006)
  being implemented. Until then, these tools return
  `{ "status": "unavailable" }` following the same pattern as edge tools.

## Alternatives Considered

1. **Separate MCP servers per domain**: Run a kernel MCP server, a swarm MCP
   server, and a research MCP server on different ports. Clean separation of
   concerns, but triples the connection management, complicates RBAC (three
   separate token stores), and forces clients to know which server handles
   which tool. Rejected.

2. **GraphQL endpoint**: Single endpoint with typed schema, supports
   subscriptions (replacing WebSocket need from ADR-008), and allows clients
   to request exactly the fields they need. However, adds `async-graphql`
   dependency, requires schema maintenance parallel to tool definitions,
   and LLM agents are better at JSON-RPC tool calling than GraphQL query
   construction. Rejected.

3. **REST endpoints**: Standard `GET /api/swarm/status`, `POST /api/agent`,
   etc. Familiar to web developers but creates a parallel API surface
   alongside MCP, doubles the routing and auth code, and LLM agents would
   need HTTP client capabilities instead of using native tool calling.
   Rejected.

## References
- `crates/rlmx-mcp/src/tools.rs` -- existing 15 tool definitions.
- `crates/rlmx-mcp/src/server.rs` -- RBAC enforcement, `handle_tool_call()`.
- `crates/rlmx-mcp/src/lib.rs` -- `McpConfig`, `Role` enum.
- `crates/rlmx-cli/src/main.rs` -- CLI entry point.
- ADR-006 -- evolutionary auto-research (research tool backends).
- ADR-008 -- WebSocket server (event streaming complement to tool calls).
