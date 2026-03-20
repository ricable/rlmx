# Plan: OpenShell Sandbox Integration — Types + ADR

## Context

The PRD `opensheell-nemoclaw.md` defines a 7-phase "Autonomous AI Agent Sandbox Factory" combining NVIDIA OpenShell (kernel-level sandboxing), SkyPilot (multi-cloud fleet), HF Jobs (GPU bursts), and ruvnet crate profiles. It's entirely Python-based. RLMX is Rust/WASM.

**Decision**: Implement only the Rust types and an ADR now. No runtime implementation yet. The types establish the vocabulary; the ADR locks the architecture. Runtime (SandboxManager, HttpCloudProvider, MCP tools) comes later.

**Strategy**: Rust-native types + thin HTTP bridge (Python stays external, talks to RLMX via MCP).

## What the PRD Brings That RLMX Doesn't Have

1. **Sandbox profiles** — deployment descriptors composing agent type + resources + isolation + model tier
2. **Fleet manifests** — declarative multi-sandbox specifications
3. **Cloud burst wiring** — the `CloudProvider` trait exists but is stubbed; the PRD shows what a real implementation looks like
4. **Network isolation intent** — no RLMX type currently expresses "this agent can only reach cluster peers"

## What RLMX Already Covers (skip these)

- Health monitoring (`HealthMonitor`), agent lifecycle (`AgentLifecycle`), zone topology, permission matrix, event bus, crate discovery (use `cargo metadata`)

## Changes

### 1. New file: `crates/rlmx-swarm/src/sandbox.rs` (~120 lines)

Core types only, no implementation logic:

```rust
// Resource envelope
pub struct ResourceEnvelope {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub gpu: GpuRequirement,
    pub disk_mb: u64,
    pub max_runtime: Duration,
}

pub enum GpuRequirement { None, Metal, Cuda { min_vram_gb: u32 }, WebGpu }

pub enum NetworkPolicy { Isolated, EgressOnly, ClusterOnly, Open }

// The core type: composes AgentType + deployment concerns
pub struct SandboxProfile {
    pub name: String,
    pub agent_type: AgentType,     // from rlmx-agents (re-export via rlmx-kernel)
    pub resources: ResourceEnvelope,
    pub network: NetworkPolicy,
    pub zone_preference: String,
    pub model_tier: ModelTier,
    pub crate_features: Vec<String>,
    pub rvf_payload: Option<String>,
}

pub struct SandboxId(pub String);

pub enum SandboxState {
    Provisioning, Starting, Running, Suspended, Stopping, Terminated, Failed,
}

pub struct SandboxInstance {
    pub id: SandboxId,
    pub profile: SandboxProfile,
    pub state: SandboxState,
    pub node_id: Option<NodeId>,
    pub vm_id: Option<VmId>,
    pub created_at: DateTime<Utc>,
    pub metrics: SandboxMetrics,
}

pub struct SandboxMetrics {
    pub cpu_usage_pct: f64,
    pub memory_usage_mb: u64,
    pub uptime_secs: u64,
    pub tasks_completed: u64,
}

// Fleet manifest
pub struct FleetManifest {
    pub name: String,
    pub version: String,
    pub sandboxes: Vec<SandboxSpec>,
    pub cloud_policy: Option<CloudPolicy>,
}

pub struct SandboxSpec {
    pub profile: String,
    pub count: usize,
    pub overrides: Option<serde_json::Value>,
}
```

### 2. Edit: `crates/rlmx-swarm/src/lib.rs`

Add `pub mod sandbox;` and re-export key types.

### 3. Edit: `crates/rlmx-swarm/src/types.rs`

Add two `SwarmEvent` variants:
- `SandboxSpawned { sandbox_id: String, profile: String, node_id: Option<String> }`
- `SandboxTerminated { sandbox_id: String, reason: String }`

### 4. New file: `docs/ADR/ADR-011-sandbox-orchestration.md`

ADR documenting:
- **Context**: Need for isolated, resource-bounded agent execution across local cluster + cloud burst
- **Decision**: Sandbox as a composition layer (SandboxProfile = AgentType + ResourceEnvelope + NetworkPolicy), not a new agent type
- **Integration strategy**: Rust types own the runtime model; Python/SkyPilot/HF Jobs stay external, connected via HttpCloudProvider implementing the existing CloudProvider trait
- **Future phases**: SandboxManager, MCP tools, CLI subcommand, dashboard view
- **Mapping**: 11 PRD profiles to existing AgentType + zone + ModelTier combinations

### 5. Edit: `crates/rlmx-swarm/Cargo.toml`

No new dependencies needed — types use existing `serde`, `chrono`, `uuid` already in the crate.

## Files Modified

| File | Action | Lines |
|------|--------|-------|
| `crates/rlmx-swarm/src/sandbox.rs` | CREATE | ~120 |
| `crates/rlmx-swarm/src/lib.rs` | EDIT | +1 line |
| `crates/rlmx-swarm/src/types.rs` | EDIT | +6 lines (2 SwarmEvent variants) |
| `docs/ADR/ADR-011-sandbox-orchestration.md` | CREATE | ~150 |

## What NOT to Do

- Do NOT port Python scripts into Rust
- Do NOT add SkyPilot/HF Jobs dependencies
- Do NOT implement SandboxManager logic yet
- Do NOT add MCP tools yet
- Do NOT modify AgentConfig or AgentType
- Do NOT add CLI subcommands yet

## Verification

1. `cargo build --workspace` — must compile clean (new types are additive)
2. `cargo test --workspace` — all 462+ tests pass (no behavior changes)
3. `cargo clippy --workspace -- -D warnings` — zero warnings
4. New types have `#[derive(Debug, Clone, Serialize, Deserialize)]` for MCP compatibility
5. ADR is consistent with existing ADR format in `docs/ADR/`
