# ADR-033: External Runtime Bridges

| Field       | Value                                         |
|-------------|-----------------------------------------------|
| Status      | Implemented                                   |
| Date        | 2026-03-19                                    |
| Deciders    | RLMX Core Team                                |
| Supersedes  | —                                             |
| Related     | ADR-029 (Universal Agent Deployment Layer)     |

## Context

RLMX agents need to interact with external LLM runtimes beyond the built-in RLMX
inference stack. Popular developer tools — Claude Code CLI, OpenAI Codex API,
Cursor IDE, and generic OpenAI-compatible HTTP endpoints — each have different
invocation patterns (subprocess CLI, REST API, proprietary protocol).

The existing `@aix/deploy` transport layer supports MCP, REST, WebSocket, and several
stub protocols but has no bridge abstraction for wrapping external LLM runtimes as
RLMX-compatible transport adapters.

## Decision

Add a **bridge** transport type to `@aix/deploy` that dispatches to runtime-specific
adapter implementations:

1. **`TransportSpec` extension** — new union member:
   `{ type: 'bridge'; runtime: 'claude-code' | 'codex' | 'cursor' | 'opencode' | 'http-generic'; config: BridgeConfig }`

2. **`BridgeConfig`** — optional `command`, `endpoint`, `model`, `authEnvVar` fields
   covering both CLI-based and HTTP-based runtimes.

3. **Adapter implementations**:
   - `ClaudeCodeBridgeAdapter` — spawns `claude` CLI subprocess, captures stdout.
   - `CodexBridgeAdapter` — HTTP POST to Codex API endpoint.
   - `HttpGenericBridgeAdapter` — HTTP POST with OpenAI-compatible request format.

4. **Factory integration** — `createTransportAdapter()` gains a `case 'bridge'` branch
   that dispatches to the correct adapter based on the `runtime` field.

5. **Templates** — four frozen bridge templates added: `claude-code-bridge`,
   `codex-bridge`, `cursor-bridge`, `generic-llm-bridge`.

## Consequences

- RLMX agents can seamlessly delegate inference to external runtimes.
- New runtimes are added by implementing `TransportAdapter` and registering in the factory.
- Bridge adapters handle errors gracefully (process spawn failures, HTTP errors).
- Health checks verify runtime availability before use.
- Templates provide one-line setup for common runtimes.
