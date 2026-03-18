# ADR-011: Sandbox Orchestration for Isolated Agent Execution

| Field    | Value                     |
|----------|---------------------------|
| Status   | Accepted                  |
| Date     | 2026-03-18                |
| Authors  | Cedric                    |
| Replaces | —                         |

## Context

RLMX agents currently run as in-process Tokio tasks sharing the host's resources with no isolation boundaries. There is no way to express "this agent needs 32 GB RAM, a Metal GPU, and may only communicate with cluster peers" as a deployable unit.

The OpenShell/NemoClaw PRD (opensheell-nemoclaw.md) identifies the need for:

1. **Kernel-level sandboxing** — Landlock + seccomp + network namespaces per agent.
2. **Resource envelopes** — CPU, memory, GPU, disk, and runtime caps.
3. **Fleet orchestration** — declarative manifests deploying multiple sandboxes across local hardware (Mac Studio M1+M3 cluster) and cloud burst (SkyPilot, HF Jobs).
4. **Cognitive specialization** — per-sandbox model tier and crate feature selection.

RLMX already covers agent lifecycle, zone topology, health monitoring, permission matrix, and event bus. The `CloudProvider` trait in `cloud.rs` is stubbed but defines the integration seam. What is missing is a **deployment descriptor layer** that composes agent type with resource/isolation/model constraints.

## Decision

### Introduce `SandboxProfile` as a composition layer

A `SandboxProfile` is **not** a new agent type. It wraps an existing `AgentType` with deployment concerns:

| Field             | Type               | Purpose                                       |
|-------------------|--------------------|-----------------------------------------------|
| `name`            | `String`           | Human-readable profile identifier              |
| `agent_type`      | `String`           | Existing AgentType variant name                |
| `resources`       | `ResourceEnvelope` | CPU, memory, GPU, disk, max runtime            |
| `network`         | `NetworkPolicy`    | Isolation level (Isolated/EgressOnly/ClusterOnly/Open) |
| `zone_preference` | `String`           | Target ZoneId for placement                    |
| `model_tier`      | `String`           | ModelTier variant name (Small/Medium/Custom)   |
| `crate_features`  | `Vec<String>`      | Feature gates to enable (e.g., `ruvllm`, `metal`) |
| `rvf_payload`     | `Option<String>`   | Optional .rvf container image path             |

### Integration strategy: Rust-native types + thin HTTP bridge

- **Rust owns the runtime model.** All sandbox types (`SandboxProfile`, `ResourceEnvelope`, `NetworkPolicy`, `SandboxId`, `SandboxState`, `SandboxInstance`, `FleetManifest`) live in `rlmx-swarm::sandbox`.
- **External orchestrators stay external.** Python/SkyPilot/HF Jobs are not imported into Rust. They communicate via MCP HTTP (port 3000) using future `rlmx_sandbox_*` tools.
- **`CloudProvider` is the integration seam.** A future `HttpCloudProvider` will implement the existing `CloudProvider` trait, bridging to SkyPilot/HF Jobs REST APIs.

### Profile-to-agent mapping

The PRD defines 11 sandbox profiles. Each maps to an existing AgentType + zone + ModelTier:

| PRD Profile           | AgentType      | Zone   | ModelTier | GPU         |
|-----------------------|----------------|--------|-----------|-------------|
| ran-optimizer         | Experimenter   | A      | Medium    | Metal       |
| hypothesis-generator  | Researcher     | A      | Medium    | None        |
| data-collector        | Worker         | C      | Small     | None        |
| model-trainer         | Experimenter   | A      | Custom    | Cuda(24GB)  |
| result-analyzer       | Analyst        | B      | Medium    | None        |
| paper-writer          | Worker         | C      | Medium    | None        |
| code-generator        | Builder        | A      | Medium    | Metal       |
| peer-reviewer         | Validator      | B      | Small     | None        |
| knowledge-curator     | Librarian      | B      | Small     | None        |
| orchestrator          | Coordinator    | A      | Medium    | None        |
| burst-worker          | Worker         | D      | Custom    | Cuda(80GB)  |

### SwarmEvent extensions

Two new variants added to `SwarmEvent`:

- `SandboxSpawned { sandbox_id, profile, node_id }` — emitted when a sandbox instance starts.
- `SandboxTerminated { sandbox_id, reason }` — emitted when a sandbox instance stops.

These integrate with the existing WebSocket event stream (ADR-008) and dashboard.

## Implemented Components

1. **SandboxManager** (`sandbox.rs`) — runtime component with 11 methods: register_profile, spawn, transition, terminate, status, list_instances, count_by_state, validate_fleet, deploy_fleet. `SandboxError` enum (5 variants). `valid_transition()` enforces legal state machine transitions.
2. **5 MCP tools** (`tools.rs`) — `rlmx_sandbox_spawn`, `rlmx_sandbox_terminate`, `rlmx_sandbox_status`, `rlmx_sandbox_list`, `rlmx_fleet_deploy`. RBAC: spawn/terminate/fleet = `ParameterModify`, status/list = `Query`. Total tools: 28.
3. **CLI subcommand** (`main.rs`) — `rlmx sandbox {spawn, terminate, status, list, fleet, profiles}`.
4. **WebSocket events** (`ws.rs`) — `SandboxSpawned`, `SandboxTerminated` variants (9 total).
5. **Dashboard views** — Sandboxes view in both `frontend/index.html` (vanilla) and `frontend/dashboard/` (Svelte).

## Future Phases

1. **HttpCloudProvider** — `CloudProvider` implementation calling SkyPilot/HF Jobs via HTTP.
2. **Kernel-level enforcement** — Landlock + seccomp + netns per sandbox (requires Linux).
3. **Resource monitoring** — Live metrics collection from running sandboxes.
4. **Auto-scaling** — Fleet auto-scale based on CloudPolicy burst thresholds.

## Consequences

### Positive

- **Additive change** — 504+ tests (16 new sandbox tests). No existing types or APIs broken (two new `SwarmEvent` variants are backward-compatible for serde).
- **Clean separation** — deployment concerns (resources, isolation, fleet) are decoupled from agent logic (lifecycle, permissions, cognitive).
- **Extensible** — new profiles can be added without code changes (they're data, not enum variants).
- **Cloud-ready** — the `CloudProvider` trait + `FleetManifest` + `CloudPolicy` provide a clear path from local Mac cluster to SkyPilot/HF Jobs burst.

### Negative

- **String-typed references** — `agent_type`, `zone_preference`, and `model_tier` are strings rather than enum variants. This trades compile-time safety for cross-crate decoupling (sandbox types don't depend on `rlmx-agents` or `rlmx-ruvllm`). Validation happens at runtime in the future `SandboxManager`.

### Risks

- Fleet manifests could drift from available profiles. Mitigation: future `SandboxManager` validates profiles at deploy time.
- Resource envelopes are advisory until enforcement is implemented. Mitigation: documented as such; kernel-level enforcement is a future phase.

## References

- `crates/rlmx-swarm/src/sandbox.rs` — type definitions
- `crates/rlmx-swarm/src/cloud.rs` — `CloudProvider` trait, `VmId`, `CloudPolicy`
- `crates/rlmx-swarm/src/types.rs` — `SwarmEvent` (extended)
- `opensheell-nemoclaw.md` — source PRD
- ADR-001 — distributed swarm architecture (zone topology)
- ADR-005 — capability-secured agents (permission matrix)
- ADR-009 — browser WASM compute pool (BrowserCompute placement)
