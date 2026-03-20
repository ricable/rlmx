# ADR-034: A2A Protocol

Status: Implemented

## Context

RLMX agents need a standardized way to discover each other's capabilities and delegate tasks across process boundaries. The existing MCP tool layer (ADR-010) handles human-to-agent and tool-to-agent interactions, but agent-to-agent communication relies on ad-hoc message passing through the kernel event bus. As the swarm grows (17 agent types, 6 zones), agents must be able to:

1. Advertise their skills to other agents (discovery).
2. Send structured task requests and receive results (delegation).
3. Track task lifecycle through well-defined states (state machine).

Google's Agent-to-Agent (A2A) protocol provides an open, JSON-RPC 2.0-based standard for exactly these interactions. Adopting A2A gives RLMX interoperability with external agent ecosystems while standardizing internal agent communication.

## Decision

Implement the A2A protocol as `rlmx-kernel::a2a` (Rust types) and `@aix/a2a` (TypeScript client/server), following these design principles:

### Agent Cards

Every RLMX agent exposes an **Agent Card** — a JSON document describing the agent's identity, capabilities, and advertised skills. The card is served at `/.well-known/agent.json` by convention.

```
AgentCard:
  name:           String
  version:        String
  description:    String
  url:            String
  capabilities:   AgentCapabilities
    streaming:              bool
    push_notifications:     bool
    state_transition_history: bool
  skills:         Vec<A2ASkill>
  authentication: AuthenticationInfo
    schemes:      Vec<String>
```

All 17 RLMX agent types are exposed as A2A skills via `AgentCardBuilder`, allowing external agents to discover and invoke any RLMX agent capability.

### Task State Machine

Tasks follow a strict state machine:

```
Submitted -> Working -> Completed
                    \-> Failed
                    \-> InputRequired -> Working (resume)
         -> Cancelled
```

Valid transitions:
- `Submitted` -> `Working`, `Cancelled`
- `Working` -> `Completed`, `Failed`, `InputRequired`, `Cancelled`
- `InputRequired` -> `Working`, `Cancelled`
- `Completed`, `Failed`, `Cancelled` are terminal states.

### Structured Messages

Messages use a parts-based structure supporting multiple content types:

- **TextPart**: Plain text content.
- **FilePart**: Base64-encoded file with MIME type.
- **DataPart**: Arbitrary structured JSON data.

### JSON-RPC Methods

| Method | Description |
|--------|-------------|
| `tasks/send` | Create or continue a task with a message |
| `tasks/get` | Retrieve current task state and messages |
| `tasks/cancel` | Request task cancellation |

### Implementation

- **Rust** (`rlmx-kernel::a2a`): Core types (`TaskState`, `A2ASkill`, `AgentCard`, `AgentCapabilities`, `AuthenticationInfo`) and `AgentCardBuilder` that maps all 17 agent types to A2A skills.
- **TypeScript** (`@aix/a2a`): Full client/server implementation with `TaskStore` (FIFO eviction at 1,000 tasks), `A2AServer` (JSON-RPC handler), `A2AClient` (remote invocation), and `buildAgentCard` discovery helper.

### TaskStore Eviction

The in-memory `TaskStore` enforces a maximum of 1,000 tasks. When the limit is reached, the oldest task (by insertion order) is evicted before the new task is stored. This prevents unbounded memory growth in long-running agent processes.

## Consequences

- All 17 RLMX agent types are discoverable via standard A2A agent cards.
- External A2A-compatible agents can delegate tasks to RLMX agents and vice versa.
- Task lifecycle is explicit and auditable through the state machine.
- The FIFO eviction policy keeps memory bounded without requiring external storage.
- Future work: streaming support (SSE), push notifications, and A2A-over-MCP bridge.
