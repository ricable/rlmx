# PRD: Autonomous AI Agent Sandbox Factory

## RVF Cognitive Containers × OpenShell × SkyPilot × HF Jobs

**Version:** 1.0  
**Date:** March 18, 2026  
**Author:** Ced  
**Status:** Draft  

---

## Executive Summary

This PRD defines a multi-phase system for spawning, specializing, and managing isolated AI agent sandboxes at scale across on-premise hardware (Mac Studio cluster), hybrid cloud, and serverless GPU infrastructure. The system combines NVIDIA OpenShell for agent isolation, SkyPilot for multi-cloud orchestration, Hugging Face Jobs for serverless GPU bursts, and the ruvnet/RuVector crate ecosystem for cognitive specialization payloads.

The core insight driving this architecture: **OpenShell is the sandbox runtime, SkyPilot is the fleet launcher, HF Jobs is the burst plane, and ruvnet crates are the specialization layer**. NemoClaw is acknowledged as an OpenClaw convenience wrapper around OpenShell, but is not the primary integration path — the system targets OpenShell directly for maximum flexibility.

---

## 1. Problem Statement

Building autonomous AI agents that run continuously requires solving four simultaneous problems:

1. **Isolation**: Agents executing arbitrary code need kernel-level sandboxing with policy-enforced filesystem, network, and process controls.
2. **Specialization**: Different agents need different cognitive toolkits — vector databases, GNN layers, witness chains, swarm coordination, sensing pipelines — without manual per-agent configuration.
3. **Scale**: Spawning tens to hundreds of agent sandboxes across heterogeneous infrastructure (local Apple Silicon, on-prem GPU, multi-cloud spot instances) requires unified orchestration.
4. **Freshness**: The ruvnet crate ecosystem publishes new capabilities monthly; sandbox images must absorb new crates automatically without manual rebuilds.

---

## 2. Component Landscape (March 2026)

### 2.1 What Ships Today

| Component | Version / Status | Role | Key Constraint |
|-----------|-----------------|------|----------------|
| **NVIDIA OpenShell** | Alpha, Apache 2.0 | Sandbox runtime: Landlock + seccomp + netns isolation, declarative YAML policies, K3s gateway, credential injection, inference routing | Single-player mode; no multi-tenant API yet |
| **NemoClaw** | Early preview | OpenClaw plugin wrapping OpenShell + Nemotron; TypeScript CLI + Python blueprint | Requires fresh OpenClaw install; single-sandbox; not a general sandbox API |
| **SkyPilot** | v0.11 (production) | Multi-cloud/K8s orchestrator: 20+ providers, spot recovery, gang scheduling, managed jobs, pools | Does not know about OpenShell natively; provisions VMs/pods only |
| **HF CLI `hf jobs`** | GA (Pro/Enterprise) | Serverless Docker execution on HF infra with GPU flavors (A10G, etc.), `hf jobs scheduled` for cron | Pay-per-second; not designed for always-on agents |
| **RuVector / RVF** | 87+ crates on crates.io | Self-booting cognitive containers (.rvf), HNSW vector DB, GNN, WASM runtime, witness chains | RVF kernel boot requires Linux; WASM tier works everywhere |
| **ruvnet ecosystem** | 82+ crates total | QuDAG, ruv-FANN, ruv-swarm-*, wifi-densepose-*, nervous-system, profiler, prime-radiant | Crates are Rust libraries, not standalone services |

### 2.2 What Does Not Exist (Glue to Build)

| Gap | Description | Phase |
|-----|-------------|-------|
| RVF → OpenShell bridge | No native integration between .rvf cognitive containers and OpenShell sandbox images | Phase 1 |
| Multi-sandbox spawner | No API for programmatically spawning N OpenShell sandboxes with different specializations | Phase 2 |
| SkyPilot → OpenShell bootstrap | SkyPilot provisions hosts; OpenShell bootstraps inside them; no integration layer exists | Phase 3 |
| Crate auto-discovery pipeline | No tooling to poll crates.io for new ruvnet crates and rebuild specialized sandbox images | Phase 5 |
| Declarative sandbox spec format | No standard manifest tying together image + crate profile + policy + model routing + telemetry | Phase 2 |

### 2.3 NemoClaw vs OpenShell: Architectural Boundary

**Critical design decision**: This system targets OpenShell directly, not NemoClaw.

NemoClaw is documented as an OpenClaw plugin split into a TypeScript CLI plus a versioned Python blueprint that creates and configures OpenShell sandboxes. NVIDIA explicitly recommends using `openshell sandbox create` directly for new installs rather than forcing the plugin path. The ruvnet crates are Rust/WASM libraries — they are not first-class NemoClaw extensions. The clean integration path is:

- Package crates into custom OpenShell sandbox images
- Expose them to agents as local CLIs, localhost HTTP services, preloaded WASM components, or OpenShell skill/tool wrappers
- Use OpenShell policy to constrain access

NemoClaw remains available as an optional convenience layer for OpenClaw-specific agents, but is not on the critical path.

---

## 3. Architecture

### 3.1 Three-Layer Model

```
┌──────────────────────────────────────────────────────────────┐
│  LAYER 3 — FLEET ORCHESTRATION                               │
│                                                              │
│  SkyPilot (primary)           HF Jobs (burst)                │
│  • Multi-cloud provisioning   • Serverless GPU execution     │
│  • Spot recovery + retries    • Scheduled cron tasks         │
│  • On-prem K8s / Slurm        • Pay-per-second overflow      │
│  • Gang scheduling             • Image rebuild pipelines      │
│  • sky exec -d for queueing   • hf jobs run / scheduled      │
├──────────────────────────────────────────────────────────────┤
│  LAYER 2 — SANDBOX RUNTIME                                   │
│                                                              │
│  NVIDIA OpenShell                                            │
│  • Container-based sandbox with K3s gateway                  │
│  • Landlock LSM + seccomp + network namespace                │
│  • Declarative YAML policy (static: fs/process; dynamic:     │
│    network/inference — hot-reloadable)                        │
│  • Credential injection via providers (never on disk)        │
│  • Inference routing: local ↔ cloud ↔ privacy router         │
│  • --from: community package | local dir | container image   │
│  • --gpu: host GPU passthrough (NVIDIA Container Toolkit)    │
├──────────────────────────────────────────────────────────────┤
│  LAYER 1 — COGNITIVE PAYLOAD                                 │
│                                                              │
│  ruvnet Crate Profiles (per-sandbox specialization)          │
│  • audited-reasoner: cognitive-container + prime-radiant     │
│  • edge-wasm-agent: router-wasm + graph-transformer-wasm     │
│  • persistent-memory: ruvector-postgres + collections        │
│  • benchmark-eval: profiler + dither                         │
│  • microvm-sealed: rvf-kernel + rvf-cli + rvf-server         │
│  • ran-optimizer: ruvector-core + nervous-system + data-fw    │
│  • swarm-mesh: qudag + qudag-network + ruv-swarm-core        │
│  • content-publisher: rvf-runtime + ruv-swarm-agents         │
│  • wifi-sensing: wifi-densepose-* (15 crates)                │
└──────────────────────────────────────────────────────────────┘
```

### 3.2 Deployment Topology

```
┌────────────────────────────────────────────────────────────────┐
│  LOCAL SITE: Mac Studio Cluster                                │
│  M1 Max (128GB) + M3 Max (128GB) = ~256GB unified memory       │
│                                                                │
│  ┌──────────────────┐     ┌──────────────────┐                 │
│  │ OpenShell GW #1  │     │ OpenShell GW #2  │                 │
│  │ (M3 Max node)    │     │ (M1 Max node)    │                 │
│  │                  │     │                  │                 │
│  │ ┌──────────────┐ │     │ ┌──────────────┐ │                 │
│  │ │ ran-optimizer│ │     │ │ content-pub  │ │                 │
│  │ │ .rvf + GNN   │ │     │ │ X/LI/Sub    │ │                 │
│  │ └──────────────┘ │     │ └──────────────┘ │                 │
│  │ ┌──────────────┐ │     │ ┌──────────────┐ │                 │
│  │ │ qudag-mesh   │ │     │ │ wifi-sense   │ │                 │
│  │ │ P2P swarm    │ │     │ │ DensePose    │ │                 │
│  │ └──────────────┘ │     │ └──────────────┘ │                 │
│  │ ┌──────────────┐ │     │ ┌──────────────┐ │                 │
│  │ │ audited-     │ │     │ │ persistent-  │ │                 │
│  │ │ reasoner     │ │     │ │ memory-agent │ │                 │
│  │ └──────────────┘ │     │ └──────────────┘ │                 │
│  └──────────────────┘     └──────────────────┘                 │
│                                                                │
│  Local inference: Nemotron / Qwen3-30B via MLX                 │
│  Inference routing: OpenShell privacy router → local first     │
└───────────────────┬────────────────────────────────────────────┘
                    │ SkyPilot multi-cloud burst
                    ▼
┌────────────────────────────────────────────────────────────────┐
│  CLOUD TIER: SkyPilot-managed spot instances                   │
│                                                                │
│  AWS / GCP / CoreWeave / Lambda / Vast.ai / Nebius             │
│  ┌──────────────────────────────────────────┐                  │
│  │ GPU Nodes (A100/H100 spot)               │                  │
│  │ • OpenShell gateway per node             │                  │
│  │ • Specialized sandbox fleets             │                  │
│  │ • Fine-tuning + batch inference          │                  │
│  │ • RVF kernel-boot tier (Linux native)    │                  │
│  │ • Managed job retry + spot recovery      │                  │
│  └──────────────────────────────────────────┘                  │
│                                                                │
│  HF Jobs (serverless burst)                                    │
│  ┌──────────────────────────────────────────┐                  │
│  │ • Weekly crate discovery + image rebuild │                  │
│  │ • Eval harness runs                      │                  │
│  │ • Synthetic data generation              │                  │
│  │ • One-shot GPU tasks (no always-on)      │                  │
│  └──────────────────────────────────────────┘                  │
└────────────────────────────────────────────────────────────────┘
```

---

## 4. Sandbox Specialization Profiles

Each profile is a named crate bundle mapped to a sandbox image variant. Profiles are the unit of specialization — an agent manifest references a profile name, and the build system resolves it to the correct image.

### 4.1 Profile Definitions

| Profile ID | Primary Crates | Use Case | GPU Required |
|------------|---------------|----------|--------------|
| `audited-reasoner` | `ruvector-cognitive-container`, `prime-radiant`, `rvf-server` | Verified reasoning with coherence gates, contradiction/safety checks, witness chains | No |
| `edge-wasm-agent` | `ruvector-router-wasm`, `ruvector-graph-transformer-wasm`, `ruvector-verified-wasm` | Browser-side or WASI agent, offline-first, edge deployment | No |
| `persistent-memory` | `ruvector-postgres`, `ruvector-collections` | Long-running agents needing durable vector memory with multi-tenancy | No (CPU sufficient) |
| `benchmark-eval` | `ruvector-profiler`, `ruvector-dither` | Memory/power/latency observability, eval harness agent | Optional |
| `microvm-sealed` | `rvf-kernel`, `rvf-cli`, `rvf-server`, `rvf-crypto` | Self-booting sealed runtime on Linux VMs, TEE attestation | No (Linux only) |
| `ran-optimizer` | `ruvector-core`, `ruvector-nervous-system`, `ruvector-data-framework`, `ruvector-dag` | Ericsson RAN KPI optimization, GNN-based cell topology modeling | Yes (A100+ preferred) |
| `swarm-mesh` | `qudag`, `qudag-network`, `qudag-crypto`, `ruv-swarm-core`, `ruv-swarm-transport` | Multi-agent swarm coordination, quantum-resistant P2P mesh | No |
| `content-publisher` | `rvf-runtime`, `ruv-swarm-agents` | Autonomous content pipeline (X, LinkedIn, Substack) | No |
| `wifi-sensing` | `wifi-densepose-*` (15 crates) | WiFi-based pose estimation, vital signs, presence detection | Optional |
| `neural-inference` | `ruvllm`, `ruv-fann`, `ruvllm-esp32` | Local LLM inference, edge neural networks | Yes |
| `full-stack` | All of the above | Development/testing sandbox with everything installed | Yes |

### 4.2 Profile Manifest Schema

Each sandbox is defined by a declarative manifest. This is the unit of reproducibility.

```yaml
# agent-manifest.yaml
apiVersion: sandbox-factory/v1
kind: AgentSandbox

metadata:
  name: ericsson-ran-optimizer-prod
  labels:
    team: ran-engineering
    environment: production

spec:
  # Cognitive payload
  profile: ran-optimizer
  crates:                          # Override or extend profile
    add:
      - ruvector-postgres          # Add persistent memory
    pin:
      ruvector-core: "2.0.4"      # Pin specific version

  # Agent runtime
  agent:
    type: claude                   # claude | openclaw | codex | custom
    provider_keys:
      - ANTHROPIC_API_KEY
      - HF_TOKEN

  # RVF payload (optional)
  rvf:
    file: ./rvf-images/ran-optimizer.rvf
    mount: /sandbox/agent.rvf
    serve_port: 8080

  # OpenShell policy
  policy:
    filesystem:
      allow: [/sandbox, /tmp, /models, /data]
    network:
      allow:
        - api.anthropic.com
        - build.nvidia.com
        - huggingface.co
        - "*.ericsson.com"         # ENM/OSS endpoints
      operator_approval: true      # Unknown hosts surfaced in TUI
    process:
      seccomp: strict
      no_new_privileges: true

  # Inference routing
  inference:
    primary:
      provider: local
      model: qwen3-30b-mlx
      endpoint: host.docker.internal:11434
    fallback:
      provider: nvidia
      model: nemotron-3-super-120b-a12b
    overflow:
      provider: anthropic
      model: claude-sonnet-4

  # Resources
  resources:
    gpu: true
    memory: 32Gi
    cpu: 8

  # Telemetry
  telemetry:
    profiler: true                 # ruvector-profiler hooks
    witness_chain: true            # rvf witness logging
    metrics_endpoint: http://host.docker.internal:9090/push
```

---

## 5. Implementation Phases

### Phase 1: Base Sandbox Image + OpenShell Integration

**Goal**: Build and validate a single RVF-enabled OpenShell sandbox locally.  
**Duration**: 1 week  
**Deliverables**: `Dockerfile.rvf-base`, `boot-rvf.sh`, validated sandbox creation

#### 1.1 OpenShell Sandbox Package Structure

OpenShell sandbox packages support: Dockerfile, optional `policy.yaml`, `skills/` directory, and startup scripts. We follow this convention:

```
sandboxes/
└── rvf-base/
    ├── Dockerfile
    ├── policy.yaml
    ├── boot-rvf.sh
    ├── skills/
    │   └── rvf-tools/
    │       └── SKILL.md           # Agent skill for RVF operations
    └── profiles/
        └── base.toml              # Default crate manifest
```

#### 1.2 Base Dockerfile

```dockerfile
# sandboxes/rvf-base/Dockerfile
FROM ubuntu:24.04 AS builder

# System dependencies
RUN apt-get update && apt-get install -y \
    curl build-essential pkg-config libssl-dev \
    protobuf-compiler libpq-dev && \
    rm -rf /var/lib/apt/lists/*

# Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    --default-toolchain stable --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

# Core RVF runtime (always present)
RUN cargo install rvf-cli --locked
RUN cargo install rvf-runtime --locked
RUN cargo install ruvector-core --locked

# Build profile-specific crates via build arg
ARG PROFILE=base
COPY profiles/${PROFILE}.toml /tmp/profile.toml

# Profile installer script
COPY install-profile.sh /tmp/install-profile.sh
RUN chmod +x /tmp/install-profile.sh && /tmp/install-profile.sh /tmp/profile.toml

# --- Runtime stage ---
FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 libpq5 curl && \
    rm -rf /var/lib/apt/lists/*

# Copy built binaries
COPY --from=builder /root/.cargo/bin/ /usr/local/bin/
COPY --from=builder /usr/local/lib/ /usr/local/lib/

# Sandbox workspace
RUN mkdir -p /sandbox /models /data /tmp
WORKDIR /sandbox

# Boot script
COPY boot-rvf.sh /usr/local/bin/boot-rvf.sh
RUN chmod +x /usr/local/bin/boot-rvf.sh

# Skills for the agent
COPY skills/ /sandbox/skills/

# Metadata
ARG PROFILE=base
LABEL org.ruvnet.sandbox.profile="${PROFILE}"
LABEL org.ruvnet.sandbox.version="1.0.0"

ENTRYPOINT ["/usr/local/bin/boot-rvf.sh"]
```

#### 1.3 Profile Installer

```bash
#!/bin/bash
# install-profile.sh — Reads a TOML profile and installs listed crates
set -euo pipefail

PROFILE_FILE="$1"

echo "=== Installing crates from profile: $PROFILE_FILE ==="

# Parse crate names from TOML (simple grep, no TOML parser needed)
grep -E '^[a-z]' "$PROFILE_FILE" | while IFS='=' read -r crate version; do
    crate=$(echo "$crate" | tr -d ' "')
    version=$(echo "$version" | tr -d ' "')
    
    if [ -n "$version" ] && [ "$version" != "*" ]; then
        echo "Installing $crate@$version"
        cargo install "$crate" --version "$version" --locked 2>/dev/null || \
        cargo install "$crate" --version "$version" || \
        echo "WARN: Failed to install $crate@$version, skipping"
    else
        echo "Installing $crate (latest)"
        cargo install "$crate" --locked 2>/dev/null || \
        cargo install "$crate" || \
        echo "WARN: Failed to install $crate, skipping"
    fi
done

echo "=== Profile installation complete ==="
```

#### 1.4 Profile TOML Examples

```toml
# profiles/base.toml
# Core runtime — always included
rvf-cli = "*"
ruvector-core = "2.0"

# profiles/audited-reasoner.toml
rvf-cli = "*"
ruvector-core = "2.0"
ruvector-cognitive-container = "*"
prime-radiant = "*"
rvf-server = "*"

# profiles/ran-optimizer.toml
rvf-cli = "*"
ruvector-core = "2.0"
ruvector-nervous-system = "*"
ruvector-data-framework = "*"
ruvector-dag = "*"
ruvector-postgres = "*"
ruvector-collections = "*"

# profiles/swarm-mesh.toml
rvf-cli = "*"
ruvector-core = "2.0"
ruv-swarm-core = "*"
ruv-swarm-agents = "*"
ruv-swarm-transport = "*"
ruv-swarm-persistence = "*"
qudag = "*"
qudag-network = "*"
qudag-crypto = "*"

# profiles/microvm-sealed.toml
rvf-cli = "*"
rvf-kernel = "*"
rvf-server = "*"
rvf-crypto = "*"
rvf-ebpf = "*"

# profiles/wifi-sensing.toml
rvf-cli = "*"
ruvector-core = "2.0"
wifi-densepose-core = "*"
wifi-densepose-config = "*"
wifi-densepose-db = "*"
wifi-densepose-signal = "*"
wifi-densepose-nn = "*"
wifi-densepose-api = "*"
wifi-densepose-hardware = "*"
wifi-densepose-ruvector = "*"

# profiles/full-stack.toml
# Everything — for development/testing only
rvf-cli = "*"
rvf-kernel = "*"
rvf-server = "*"
rvf-crypto = "*"
rvf-runtime = "*"
ruvector-core = "2.0"
ruvector-cognitive-container = "*"
ruvector-nervous-system = "*"
ruvector-data-framework = "*"
ruvector-dag = "*"
ruvector-postgres = "*"
ruvector-collections = "*"
ruvector-profiler = "*"
prime-radiant = "*"
ruv-swarm-core = "*"
ruv-swarm-agents = "*"
qudag = "*"
qudag-network = "*"
```

#### 1.5 Boot Script

```bash
#!/bin/bash
# boot-rvf.sh — Sandbox entrypoint
set -euo pipefail

echo "[boot-rvf] Starting sandbox..."
echo "[boot-rvf] Profile: ${SANDBOX_PROFILE:-base}"

# 1. If an .rvf file is mounted, serve it
if [ -f /sandbox/agent.rvf ]; then
    echo "[boot-rvf] Found agent.rvf, inspecting..."
    rvf inspect /sandbox/agent.rvf 2>/dev/null || true
    
    echo "[boot-rvf] Starting RVF server on :8080..."
    rvf serve /sandbox/agent.rvf --port 8080 &
    RVF_PID=$!
    echo "[boot-rvf] RVF server PID: $RVF_PID"
fi

# 2. If profiler is available and enabled, start it
if command -v ruvector-profiler &>/dev/null && [ "${ENABLE_PROFILER:-false}" = "true" ]; then
    echo "[boot-rvf] Starting profiler..."
    ruvector-profiler --output /tmp/metrics.csv &
fi

# 3. Export tool endpoints for the agent
export RVF_ENDPOINT="http://localhost:8080"
export RUVECTOR_DB_PATH="/data/vectors.db"

# 4. Hand off to whatever OpenShell launches (claude, openclaw, codex, etc.)
echo "[boot-rvf] Handing off to agent: $*"
exec "$@"
```

#### 1.6 Validation Steps

```bash
# Build the base image
docker build -t rvf-sandbox:base -f sandboxes/rvf-base/Dockerfile \
    --build-arg PROFILE=base sandboxes/rvf-base/

# Build a specialized image
docker build -t rvf-sandbox:ran-optimizer -f sandboxes/rvf-base/Dockerfile \
    --build-arg PROFILE=ran-optimizer sandboxes/rvf-base/

# Test locally without OpenShell
docker run --rm -it rvf-sandbox:base rvf --version
docker run --rm -it rvf-sandbox:base ruvector-core --help

# Test with OpenShell
openshell sandbox create --from rvf-sandbox:base --name test-base -- claude
openshell sandbox list
openshell term test-base

# Validate policy
openshell policy show test-base
```

**Phase 1 Exit Criteria**:
- [ ] `rvf-sandbox:base` builds in < 15 minutes
- [ ] At least 3 profile variants build successfully
- [ ] `openshell sandbox create --from rvf-sandbox:ran-optimizer` succeeds
- [ ] Agent inside sandbox can call `rvf inspect` and `rvf serve`
- [ ] OpenShell policy blocks unauthorized network egress

---

### Phase 2: Declarative Sandbox Spawner

**Goal**: Programmatically spawn multiple specialized sandboxes from a fleet manifest.  
**Duration**: 1 week  
**Deliverables**: `spawn-fleet.py`, `fleet-manifest.yaml`, CLI wrapper

#### 2.1 Fleet Manifest

```yaml
# fleet-manifest.yaml
apiVersion: sandbox-factory/v1
kind: FleetManifest

metadata:
  name: ced-agent-fleet-v1
  description: "Production agent fleet for RAN optimization + content pipeline"

defaults:
  agent: claude
  policy: ./policies/baseline.yaml
  inference:
    primary:
      provider: local
      endpoint: host.docker.internal:11434
    fallback:
      provider: anthropic

sandboxes:
  - name: ran-optimizer-01
    profile: ran-optimizer
    rvf_file: ./rvf-images/ran-optimizer.rvf
    gpu: true
    resources:
      memory: 32Gi
      cpu: 8
    env:
      ENM_HOST: "enm-prod.ericsson.local"
      PM_GRANULARITY: "15min"
    policy_overrides:
      network:
        add_allow:
          - "*.ericsson.local"
          - "enm-prod.ericsson.local:443"

  - name: ran-optimizer-02
    profile: ran-optimizer
    rvf_file: ./rvf-images/ran-optimizer.rvf
    gpu: true
    resources:
      memory: 32Gi
      cpu: 8
    env:
      ENM_HOST: "enm-staging.ericsson.local"

  - name: content-publisher-x
    profile: content-publisher
    agent: openclaw
    gpu: false
    resources:
      memory: 4Gi
      cpu: 2
    env:
      PLATFORM: "twitter"
      PUBLISH_SCHEDULE: "0 8,12,18 * * *"
    policy_overrides:
      network:
        add_allow:
          - "api.twitter.com"
          - "api.x.com"

  - name: content-publisher-linkedin
    profile: content-publisher
    agent: openclaw
    gpu: false
    env:
      PLATFORM: "linkedin"

  - name: content-publisher-substack
    profile: content-publisher
    agent: openclaw
    gpu: false
    env:
      PLATFORM: "substack"

  - name: audited-reasoner-01
    profile: audited-reasoner
    gpu: false
    resources:
      memory: 8Gi
    env:
      COHERENCE_MIN_SCORE: "0.85"
      CONTRADICTION_RATE_MAX: "0.02"

  - name: qudag-mesh-01
    profile: swarm-mesh
    gpu: false
    env:
      QUDAG_PEERS: "qudag-mesh-02,qudag-mesh-03"
      QUDAG_DARK_ADDRESS: "auto"

  - name: qudag-mesh-02
    profile: swarm-mesh
    gpu: false
    env:
      QUDAG_PEERS: "qudag-mesh-01,qudag-mesh-03"

  - name: qudag-mesh-03
    profile: swarm-mesh
    gpu: false
    env:
      QUDAG_PEERS: "qudag-mesh-01,qudag-mesh-02"
```

#### 2.2 Fleet Spawner

```python
#!/usr/bin/env python3
"""spawn-fleet.py — Spawn an OpenShell sandbox fleet from a declarative manifest."""

import yaml
import subprocess
import sys
import os
import copy
from pathlib import Path
from typing import Optional


def load_manifest(path: str) -> dict:
    with open(path) as f:
        return yaml.safe_load(f)


def merge_policy(base_policy_path: str, overrides: Optional[dict]) -> str:
    """Merge base policy with per-sandbox overrides, write to temp file."""
    with open(base_policy_path) as f:
        policy = yaml.safe_load(f)
    
    if not overrides:
        return base_policy_path
    
    merged = copy.deepcopy(policy)
    
    # Merge network allow lists
    if "network" in overrides:
        if "add_allow" in overrides["network"]:
            if "allow" not in merged.get("network", {}):
                merged.setdefault("network", {})["allow"] = []
            merged["network"]["allow"].extend(overrides["network"]["add_allow"])
    
    # Write merged policy
    out_path = f"/tmp/policy-merged-{os.getpid()}.yaml"
    with open(out_path, "w") as f:
        yaml.dump(merged, f)
    return out_path


def resolve_image(profile: str) -> str:
    """Map profile name to Docker image tag."""
    tag = f"rvf-sandbox:{profile}"
    # Check if image exists locally
    result = subprocess.run(
        ["docker", "image", "inspect", tag],
        capture_output=True
    )
    if result.returncode != 0:
        print(f"  Image {tag} not found, building...")
        subprocess.run([
            "docker", "build",
            "-t", tag,
            "-f", "sandboxes/rvf-base/Dockerfile",
            "--build-arg", f"PROFILE={profile}",
            "sandboxes/rvf-base/"
        ], check=True)
    return tag


def spawn_sandbox(sandbox_spec: dict, defaults: dict) -> bool:
    """Create a single OpenShell sandbox from spec."""
    name = sandbox_spec["name"]
    profile = sandbox_spec.get("profile", "base")
    agent = sandbox_spec.get("agent", defaults.get("agent", "claude"))
    gpu = sandbox_spec.get("gpu", False)
    rvf_file = sandbox_spec.get("rvf_file")
    env_vars = sandbox_spec.get("env", {})
    policy_overrides = sandbox_spec.get("policy_overrides")
    
    print(f"\n{'='*60}")
    print(f"Spawning: {name} (profile={profile}, agent={agent}, gpu={gpu})")
    print(f"{'='*60}")
    
    # Resolve image
    image = resolve_image(profile)
    
    # Build command
    cmd = ["openshell", "sandbox", "create", "--from", image, "--name", name]
    
    if gpu:
        cmd.append("--gpu")
    
    # Agent type
    cmd.extend(["--", agent])
    
    # Execute
    try:
        subprocess.run(cmd, check=True)
    except subprocess.CalledProcessError as e:
        print(f"  ERROR: Failed to create sandbox {name}: {e}")
        return False
    
    # Apply policy
    base_policy = defaults.get("policy", "./policies/baseline.yaml")
    if os.path.exists(base_policy):
        merged_policy = merge_policy(base_policy, policy_overrides)
        subprocess.run([
            "openshell", "policy", "set", name, "--file", merged_policy
        ], check=True)
    
    # Set environment variables
    for key, value in env_vars.items():
        subprocess.run([
            "openshell", "sandbox", "exec", name,
            "--", "bash", "-c", f"export {key}='{value}'"
        ], check=False)  # Best-effort env injection
    
    print(f"  ✓ Sandbox {name} created successfully")
    return True


def main():
    if len(sys.argv) < 2:
        print("Usage: spawn-fleet.py <fleet-manifest.yaml>")
        sys.exit(1)
    
    manifest = load_manifest(sys.argv[1])
    defaults = manifest.get("defaults", {})
    sandboxes = manifest.get("sandboxes", [])
    
    print(f"Fleet: {manifest['metadata']['name']}")
    print(f"Sandboxes to spawn: {len(sandboxes)}")
    
    results = {"success": 0, "failed": 0}
    
    for spec in sandboxes:
        if spawn_sandbox(spec, defaults):
            results["success"] += 1
        else:
            results["failed"] += 1
    
    print(f"\n{'='*60}")
    print(f"Fleet spawn complete: {results['success']} success, {results['failed']} failed")
    print(f"{'='*60}")
    
    # List all sandboxes
    subprocess.run(["openshell", "sandbox", "list"])


if __name__ == "__main__":
    main()
```

#### 2.3 CLI Wrapper

```bash
#!/bin/bash
# sandbox-factory — CLI for the agent sandbox factory
set -euo pipefail

COMMAND="${1:-help}"
shift || true

case "$COMMAND" in
    spawn)
        python3 spawn-fleet.py "$@"
        ;;
    build)
        # Build one or all profiles
        PROFILE="${1:-all}"
        if [ "$PROFILE" = "all" ]; then
            for p in sandboxes/rvf-base/profiles/*.toml; do
                name=$(basename "$p" .toml)
                echo "Building profile: $name"
                docker build -t "rvf-sandbox:$name" \
                    -f sandboxes/rvf-base/Dockerfile \
                    --build-arg "PROFILE=$name" \
                    sandboxes/rvf-base/
            done
        else
            docker build -t "rvf-sandbox:$PROFILE" \
                -f sandboxes/rvf-base/Dockerfile \
                --build-arg "PROFILE=$PROFILE" \
                sandboxes/rvf-base/
        fi
        ;;
    list)
        openshell sandbox list
        ;;
    status)
        for sb in $(openshell sandbox list --format json 2>/dev/null | jq -r '.[].name'); do
            echo "$sb: $(openshell sandbox status "$sb" 2>/dev/null || echo 'unknown')"
        done
        ;;
    destroy)
        FLEET_FILE="${1:-}"
        if [ -n "$FLEET_FILE" ]; then
            names=$(python3 -c "
import yaml, sys
with open('$FLEET_FILE') as f:
    m = yaml.safe_load(f)
for s in m.get('sandboxes', []):
    print(s['name'])
")
            for name in $names; do
                echo "Destroying: $name"
                openshell sandbox destroy "$name" 2>/dev/null || true
            done
        else
            echo "Usage: sandbox-factory destroy <fleet-manifest.yaml>"
        fi
        ;;
    help|*)
        echo "sandbox-factory — AI Agent Sandbox Factory"
        echo ""
        echo "Commands:"
        echo "  spawn <manifest.yaml>    Spawn fleet from manifest"
        echo "  build [profile|all]      Build sandbox images"
        echo "  list                     List running sandboxes"
        echo "  status                   Show fleet status"
        echo "  destroy <manifest.yaml>  Tear down fleet"
        ;;
esac
```

**Phase 2 Exit Criteria**:
- [ ] `sandbox-factory spawn fleet-manifest.yaml` creates all 9 sandboxes from example manifest
- [ ] Each sandbox runs the correct agent type (claude/openclaw/codex)
- [ ] Policy overrides merge correctly (RAN agents can reach `*.ericsson.local`)
- [ ] `sandbox-factory destroy fleet-manifest.yaml` tears down cleanly
- [ ] QuDAG mesh sandboxes can discover each other's endpoints

---

### Phase 3: SkyPilot Integration for Multi-Cloud + On-Prem

**Goal**: Launch sandbox fleets on remote infrastructure via SkyPilot.  
**Duration**: 2 weeks  
**Deliverables**: SkyPilot YAML templates, remote gateway bootstrap, hybrid topology

#### 3.1 SkyPilot Task: Single-Node Fleet

```yaml
# skypilot/single-node-fleet.yaml
name: agent-fleet-single

resources:
  accelerators: A100:1
  use_spot: true
  any_of:
    - cloud: aws
      region: us-east-1
    - cloud: gcp
      region: us-central1
    - cloud: lambda
    - cloud: vast

disk_size: 200  # GB, for Docker images + models

file_mounts:
  /fleet/manifest.yaml: ./fleet-manifest.yaml
  /fleet/policies/: ./policies/
  /fleet/rvf-images/: ./rvf-images/
  /fleet/sandboxes/: ./sandboxes/
  /fleet/spawn-fleet.py: ./spawn-fleet.py

setup: |
  set -euo pipefail
  
  # Install Docker if not present
  if ! command -v docker &>/dev/null; then
      curl -fsSL https://get.docker.com | sh
      sudo usermod -aG docker $USER
      newgrp docker
  fi
  
  # Install OpenShell
  curl -fsSL https://raw.githubusercontent.com/NVIDIA/OpenShell/main/install.sh | bash
  
  # Install NVIDIA Container Toolkit (for GPU passthrough)
  if command -v nvidia-smi &>/dev/null; then
      distribution=$(. /etc/os-release; echo $ID$VERSION_ID)
      curl -fsSL https://nvidia.github.io/libnvidia-container/gpgkey | \
          sudo gpg --dearmor -o /usr/share/keyrings/nvidia-container-toolkit-keyring.gpg
      curl -s -L "https://nvidia.github.io/libnvidia-container/$distribution/libnvidia-container.list" | \
          sed 's#deb https://#deb [signed-by=/usr/share/keyrings/nvidia-container-toolkit-keyring.gpg] https://#g' | \
          sudo tee /etc/apt/sources.list.d/nvidia-container-toolkit.list
      sudo apt-get update && sudo apt-get install -y nvidia-container-toolkit
      sudo nvidia-ctk runtime configure --runtime=docker
      sudo systemctl restart docker
  fi
  
  # Build sandbox images
  cd /fleet
  for profile in sandboxes/rvf-base/profiles/*.toml; do
      name=$(basename "$profile" .toml)
      echo "Building rvf-sandbox:$name"
      docker build -t "rvf-sandbox:$name" \
          -f sandboxes/rvf-base/Dockerfile \
          --build-arg "PROFILE=$name" \
          sandboxes/rvf-base/ || echo "WARN: Failed to build $name"
  done
  
  pip install pyyaml

run: |
  cd /fleet
  
  # Create OpenShell gateway
  openshell gateway create
  
  # Spawn fleet
  python3 spawn-fleet.py manifest.yaml
  
  # Keep alive — SkyPilot will autostop after idle timeout
  echo "Fleet running. Sandboxes:"
  openshell sandbox list
  
  # Tail all sandbox logs
  for sb in $(openshell sandbox list --format names 2>/dev/null); do
      openshell sandbox logs "$sb" --follow &
  done
  wait
```

#### 3.2 SkyPilot Task: Multi-Node Fleet with Spot Recovery

```yaml
# skypilot/multi-node-fleet.yaml
name: agent-fleet-distributed

num_nodes: 4

resources:
  accelerators: A100:1
  use_spot: true
  any_of:
    - cloud: aws
    - cloud: gcp
    - cloud: coreweave
    - cloud: lambda
    - cloud: vast

file_mounts:
  /fleet/: ./

setup: |
  # Same as single-node setup (Docker + OpenShell + NVIDIA toolkit)
  curl -fsSL https://get.docker.com | sh 2>/dev/null || true
  curl -fsSL https://raw.githubusercontent.com/NVIDIA/OpenShell/main/install.sh | bash
  pip install pyyaml
  
  # Build images (each node builds its own)
  cd /fleet
  for profile in sandboxes/rvf-base/profiles/*.toml; do
      name=$(basename "$profile" .toml)
      docker build -t "rvf-sandbox:$name" \
          -f sandboxes/rvf-base/Dockerfile \
          --build-arg "PROFILE=$name" \
          sandboxes/rvf-base/ 2>/dev/null || true
  done

run: |
  NODE_RANK=${SKYPILOT_NODE_RANK}
  NUM_NODES=${SKYPILOT_NUM_NODES}
  MASTER_IP=$(echo "$SKYPILOT_NODE_IPS" | head -n1)
  
  echo "Node $NODE_RANK of $NUM_NODES (master: $MASTER_IP)"
  
  # Each node runs an OpenShell gateway
  openshell gateway create
  
  # Distribute sandboxes across nodes using round-robin
  cd /fleet
  python3 -c "
import yaml, subprocess, os

node_rank = int(os.environ['NODE_RANK'])
num_nodes = int(os.environ['NUM_NODES'])

with open('manifest.yaml') as f:
    manifest = yaml.safe_load(f)

sandboxes = manifest.get('sandboxes', [])
my_sandboxes = [s for i, s in enumerate(sandboxes) if i % num_nodes == node_rank]

print(f'Node {node_rank}: assigned {len(my_sandboxes)} of {len(sandboxes)} sandboxes')

for s in my_sandboxes:
    print(f'  - {s[\"name\"]} (profile={s.get(\"profile\", \"base\")})')
" 
  
  # Spawn this node's slice of the fleet
  python3 spawn-fleet.py manifest.yaml --node-rank $NODE_RANK --num-nodes $NUM_NODES
  
  # Keep alive
  wait
```

#### 3.3 On-Prem: Mac Studio Integration

SkyPilot supports Kubernetes as a backend. For the Mac Studio cluster:

```bash
# Register Mac Studio K8s cluster (via Lima + k3s or OrbStack)
sky check --cloud kubernetes

# Launch fleet on local cluster
sky launch skypilot/single-node-fleet.yaml --cloud kubernetes

# Or use SkyPilot exec to queue jobs onto existing cluster
sky exec my-cluster -- bash /fleet/spawn-fleet.py manifest.yaml
```

For direct local deployment (no SkyPilot):

```bash
# Direct local deployment on Mac Studio
# One gateway per machine
ssh m3-max "openshell gateway create"
ssh m1-max "openshell gateway create"

# Spawn sandboxes targeting specific gateways
# M3 Max: GPU-heavy agents
openshell sandbox create --remote m3-max --from rvf-sandbox:ran-optimizer \
    --name ran-optimizer-01 -- claude

# M1 Max: CPU agents
openshell sandbox create --remote m1-max --from rvf-sandbox:content-publisher \
    --name content-pub-x -- openclaw
```

#### 3.4 Hybrid Burst Pattern

```bash
# Local fleet for always-on agents
sandbox-factory spawn fleet-manifest-local.yaml

# Cloud burst for GPU-heavy batch work
sky jobs launch -n ran-eval-batch skypilot/single-node-fleet.yaml

# HF Jobs for one-shot tasks
hf jobs run --flavor a10g-small rvf-sandbox:ran-optimizer \
    python3 /sandbox/run_eval.py --dataset ericsson-pm-2026q1
```

**Phase 3 Exit Criteria**:
- [ ] `sky launch single-node-fleet.yaml` provisions a cloud node and spawns sandboxes
- [ ] Spot preemption triggers SkyPilot retry on a different provider
- [ ] Local Mac Studio runs sandboxes via `openshell sandbox create --remote`
- [ ] Hybrid pattern works: local always-on + cloud burst via SkyPilot + HF Jobs one-shot

---

### Phase 4: Inference Routing Layer

**Goal**: Configure hybrid inference: local MLX → NVIDIA cloud → Anthropic overflow.  
**Duration**: 1 week  
**Deliverables**: OpenShell inference config, local model setup, privacy routing

#### 4.1 OpenShell Inference Configuration

OpenShell intercepts every inference call from sandboxed agents and routes it through the gateway. This is hot-reloadable without sandbox restart.

```yaml
# policies/inference-hybrid.yaml
# Included in sandbox policy via the inference section

inference:
  # Primary: local model (fastest, most private)
  primary:
    provider: local
    model: qwen3-30b-q4
    endpoint: http://host.docker.internal:11434  # Ollama / llama.cpp on host
    timeout_ms: 30000
    
  # Secondary: NVIDIA cloud (good quality, moderate latency)
  fallback:
    provider: nvidia
    model: nemotron-3-super-120b-a12b
    endpoint: https://build.nvidia.com
    api_key_env: NVIDIA_API_KEY
    
  # Overflow: Anthropic (highest quality, highest cost)
  overflow:
    provider: anthropic
    model: claude-sonnet-4
    api_key_env: ANTHROPIC_API_KEY

  # Routing rules
  routing:
    # Use local for short prompts, cloud for complex reasoning
    strategy: latency-aware
    local_max_tokens: 4096
    fallback_on_local_timeout: true
    overflow_on_fallback_error: true
```

#### 4.2 Local Model Setup (Mac Studio)

```bash
# On M3 Max: serve Qwen3-30B via MLX for local inference
pip install mlx-lm
mlx_lm.server --model mlx-community/Qwen3-30B-A3B-4bit --port 11434 &

# Or via Ollama
ollama serve &
ollama pull qwen3:30b

# Verify
curl http://localhost:11434/v1/chat/completions \
    -d '{"model":"qwen3:30b","messages":[{"role":"user","content":"test"}]}'
```

**Phase 4 Exit Criteria**:
- [ ] Agent in sandbox makes API call → routed to local MLX model
- [ ] Local timeout → automatic fallback to NVIDIA cloud
- [ ] NVIDIA error → automatic overflow to Anthropic
- [ ] `openshell inference set` hot-reloads without sandbox restart

---

### Phase 5: Automated Crate Discovery + Image Rebuild Pipeline

**Goal**: Automatically detect new ruvnet crates and rebuild specialized sandbox images.  
**Duration**: 1 week  
**Deliverables**: `discover-crates.py`, HF scheduled job, image registry push

#### 5.1 Crate Discovery Script

```python
#!/usr/bin/env python3
"""discover-crates.py — Poll crates.io for new ruvnet crates,
categorize them, update profile TOMLs, rebuild images."""

import requests
import json
import toml
import subprocess
import os
from datetime import datetime, timedelta
from pathlib import Path
from collections import defaultdict

RUVNET_API = "https://crates.io/api/v1/crates"
USER_AGENT = "sandbox-factory/1.0 (agent-sandbox-builder)"
LOOKBACK_DAYS = 90
PROFILES_DIR = Path("sandboxes/rvf-base/profiles")
REGISTRY = os.environ.get("IMAGE_REGISTRY", "localhost:5000")

# Category rules: prefix → profile
CATEGORY_RULES = [
    ("wifi-densepose-", "wifi-sensing"),
    ("rvf-", "microvm-sealed"),
    ("ruvector-cognitive", "audited-reasoner"),
    ("ruvector-postgres", "persistent-memory"),
    ("ruvector-collection", "persistent-memory"),
    ("ruvector-profiler", "benchmark-eval"),
    ("ruvector-dither", "benchmark-eval"),
    ("ruvector-router-wasm", "edge-wasm-agent"),
    ("ruvector-graph-transformer-wasm", "edge-wasm-agent"),
    ("ruvector-verified-wasm", "edge-wasm-agent"),
    ("ruvector-nervous", "ran-optimizer"),
    ("ruvector-data-framework", "ran-optimizer"),
    ("ruvector-dag", "ran-optimizer"),
    ("ruvector-core", "base"),  # Goes into base profile
    ("ruvector-", "base"),  # Catch-all ruvector
    ("prime-radiant", "audited-reasoner"),
    ("ruv-swarm-", "swarm-mesh"),
    ("qudag", "swarm-mesh"),
    ("ruv-fann", "neural-inference"),
    ("ruvllm", "neural-inference"),
]


def fetch_ruvnet_crates() -> list:
    """Fetch all crates by ruvnet user."""
    all_crates = []
    page = 1
    while True:
        resp = requests.get(
            RUVNET_API,
            params={"user_id": "ruvnet", "page": page, "per_page": 100, "sort": "new"},
            headers={"User-Agent": USER_AGENT},
        )
        resp.raise_for_status()
        data = resp.json()
        crates = data.get("crates", [])
        if not crates:
            break
        all_crates.extend(crates)
        page += 1
    return all_crates


def filter_recent(crates: list, days: int) -> list:
    cutoff = datetime.utcnow() - timedelta(days=days)
    return [
        c for c in crates
        if datetime.fromisoformat(c["updated_at"].replace("Z", "+00:00")).replace(tzinfo=None) > cutoff
    ]


def categorize(crate_name: str) -> str:
    for prefix, profile in CATEGORY_RULES:
        if crate_name.startswith(prefix):
            return profile
    return "base"


def update_profiles(new_crates: list) -> dict:
    """Add newly discovered crates to the appropriate profile TOMLs."""
    updates = defaultdict(list)
    
    for crate in new_crates:
        name = crate["id"]
        version = crate.get("max_version", "*")
        profile = categorize(name)
        updates[profile].append((name, version))
    
    changed_profiles = set()
    
    for profile, crate_list in updates.items():
        profile_path = PROFILES_DIR / f"{profile}.toml"
        
        if profile_path.exists():
            existing = toml.load(profile_path)
        else:
            existing = {"rvf-cli": "*", "ruvector-core": "2.0"}
        
        for crate_name, version in crate_list:
            if crate_name not in existing:
                print(f"  NEW: {crate_name}@{version} → {profile}")
                existing[crate_name] = "*"  # Use latest
                changed_profiles.add(profile)
        
        with open(profile_path, "w") as f:
            toml.dump(existing, f)
    
    return dict(updates)


def rebuild_images(profiles: set):
    """Rebuild Docker images for changed profiles."""
    for profile in profiles:
        tag = f"{REGISTRY}/rvf-sandbox:{profile}"
        print(f"\nRebuilding: {tag}")
        result = subprocess.run([
            "docker", "build",
            "-t", tag,
            "-f", "sandboxes/rvf-base/Dockerfile",
            "--build-arg", f"PROFILE={profile}",
            "sandboxes/rvf-base/"
        ])
        if result.returncode == 0:
            print(f"  ✓ Built {tag}")
            # Push to registry
            subprocess.run(["docker", "push", tag])
        else:
            print(f"  ✗ Failed to build {tag}")


def main():
    print("=== Crate Discovery Pipeline ===")
    print(f"Looking back {LOOKBACK_DAYS} days\n")
    
    all_crates = fetch_ruvnet_crates()
    print(f"Total ruvnet crates: {len(all_crates)}")
    
    recent = filter_recent(all_crates, LOOKBACK_DAYS)
    print(f"Updated in last {LOOKBACK_DAYS} days: {len(recent)}\n")
    
    if not recent:
        print("No new crates found. Nothing to do.")
        return
    
    # Categorize and update profiles
    updates = update_profiles(recent)
    
    # Summary
    print("\n=== Discovery Summary ===")
    changed = set()
    for profile, crates in updates.items():
        print(f"\n{profile}:")
        for name, version in crates:
            print(f"  {name} @ {version}")
        changed.add(profile)
    
    # Also always rebuild full-stack
    changed.add("full-stack")
    
    # Rebuild changed images
    print(f"\n=== Rebuilding {len(changed)} images ===")
    rebuild_images(changed)
    
    # Write discovery report
    report = {
        "timestamp": datetime.utcnow().isoformat(),
        "total_crates": len(all_crates),
        "recent_crates": len(recent),
        "profiles_updated": list(changed),
        "new_crates": {p: [(n, v) for n, v in cs] for p, cs in updates.items()},
    }
    with open("discovery-report.json", "w") as f:
        json.dump(report, f, indent=2)
    print(f"\nReport written to discovery-report.json")


if __name__ == "__main__":
    main()
```

#### 5.2 Scheduled Execution

```bash
# Weekly discovery via HF Jobs
hf jobs scheduled create \
    --cron "0 2 * * 0" \
    --flavor cpu-basic \
    --secrets HF_TOKEN \
    --secrets IMAGE_REGISTRY_TOKEN \
    rvf-sandbox:base \
    python3 /fleet/discover-crates.py

# Or via SkyPilot managed job
sky jobs launch -n crate-discovery --use-spot skypilot/crate-discovery.yaml
```

#### 5.3 Notification and Rollout

When new crates are discovered, the pipeline:
1. Updates profile TOMLs in the repository
2. Rebuilds affected Docker images
3. Pushes to the container registry
4. Writes `discovery-report.json`
5. (Future) Triggers rolling sandbox updates via `openshell sandbox update`

**Phase 5 Exit Criteria**:
- [ ] `discover-crates.py` fetches all ruvnet crates and categorizes correctly
- [ ] New crates from the last 3 months are added to appropriate profiles
- [ ] Changed profile images rebuild successfully
- [ ] HF scheduled job runs weekly without intervention
- [ ] Discovery report JSON is parseable and complete

---

### Phase 6: Observability + Health Monitoring

**Goal**: Fleet-wide visibility into sandbox health, resource usage, and agent behavior.  
**Duration**: 1 week  
**Deliverables**: Health check script, Prometheus metrics integration, witness chain aggregation

#### 6.1 Fleet Health Monitor

```python
#!/usr/bin/env python3
"""fleet-health.py — Monitor sandbox fleet health."""

import subprocess
import json
import time
import sys


def get_sandbox_list() -> list:
    """Get list of running sandboxes."""
    result = subprocess.run(
        ["openshell", "sandbox", "list", "--format", "json"],
        capture_output=True, text=True
    )
    if result.returncode != 0:
        return []
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError:
        return []


def check_sandbox_health(name: str) -> dict:
    """Check individual sandbox health."""
    health = {"name": name, "status": "unknown", "checks": {}}
    
    # Status check
    result = subprocess.run(
        ["openshell", "sandbox", "status", name],
        capture_output=True, text=True
    )
    health["status"] = result.stdout.strip() if result.returncode == 0 else "error"
    
    # RVF server check (if applicable)
    result = subprocess.run(
        ["openshell", "sandbox", "exec", name, "--",
         "curl", "-s", "-o", "/dev/null", "-w", "%{http_code}",
         "http://localhost:8080/health"],
        capture_output=True, text=True
    )
    health["checks"]["rvf_server"] = result.stdout.strip() == "200"
    
    # Policy check
    result = subprocess.run(
        ["openshell", "policy", "show", name],
        capture_output=True, text=True
    )
    health["checks"]["policy_loaded"] = result.returncode == 0
    
    return health


def monitor_loop(interval: int = 60):
    """Continuous monitoring loop."""
    while True:
        sandboxes = get_sandbox_list()
        print(f"\n{'='*60}")
        print(f"Fleet Status — {time.strftime('%Y-%m-%d %H:%M:%S')}")
        print(f"Active sandboxes: {len(sandboxes)}")
        print(f"{'='*60}")
        
        for sb in sandboxes:
            name = sb.get("name", sb) if isinstance(sb, dict) else sb
            health = check_sandbox_health(name)
            status_icon = "✓" if health["status"] == "running" else "✗"
            rvf_icon = "✓" if health["checks"].get("rvf_server") else "—"
            print(f"  {status_icon} {name:30s} | RVF: {rvf_icon} | Policy: {'✓' if health['checks'].get('policy_loaded') else '✗'}")
        
        time.sleep(interval)


if __name__ == "__main__":
    interval = int(sys.argv[1]) if len(sys.argv) > 1 else 60
    monitor_loop(interval)
```

**Phase 6 Exit Criteria**:
- [ ] `fleet-health.py` reports status of all active sandboxes
- [ ] RVF server health checks pass for sandboxes with mounted .rvf files
- [ ] Monitoring runs continuously with configurable interval
- [ ] Failed sandboxes are flagged clearly in output

---

### Phase 7: RAN-Specific Integration

**Goal**: Wire up Ericsson-specific sandbox capabilities for the RAN optimization use case.  
**Duration**: 2 weeks  
**Deliverables**: RAN sandbox profile, ENM data pipeline, KPI eval harness

#### 7.1 RAN Optimizer Sandbox Spec

```yaml
# agents/ran-optimizer.yaml
apiVersion: sandbox-factory/v1
kind: AgentSandbox

metadata:
  name: ericsson-ran-optimizer
  labels:
    domain: telecom
    vendor: ericsson
    capability: ran-optimization

spec:
  profile: ran-optimizer
  
  crates:
    add:
      - ruvector-postgres      # Persistent KPI storage
      - ruvector-profiler      # Performance monitoring
    pin:
      ruvector-core: "2.0.4"
  
  agent:
    type: claude
    provider_keys:
      - ANTHROPIC_API_KEY
  
  rvf:
    file: ./rvf-images/ran-optimizer.rvf
    mount: /sandbox/agent.rvf
    serve_port: 8080
  
  policy:
    filesystem:
      allow:
        - /sandbox
        - /tmp
        - /models
        - /data/pm              # PM counters (15-min granularity)
        - /data/fm              # Fault management exports
        - /data/cm              # Configuration management
    network:
      allow:
        - api.anthropic.com
        - "enm-*.ericsson.local"
        - "oss-*.ericsson.local"
      operator_approval: true
    process:
      seccomp: strict
  
  inference:
    primary:
      provider: local
      model: qwen3-30b-q4
      endpoint: host.docker.internal:11434
    fallback:
      provider: anthropic
      model: claude-sonnet-4
  
  env:
    # Ericsson-specific
    ENM_HOST: "enm-prod.ericsson.local"
    PM_GRANULARITY: "15min"
    KPI_TARGETS: "accessibility>99.5,retainability>99.8,cssr>99.9"
    
    # Feature optimization scope
    FEATURES: "IFLB,DUAC,MCPC,ANR,CarrierAggregation"
    PARAMETERS: "pZeroNominalPusch,alpha,cellIndividualOffsetEUtran"
    
    # RVF settings
    ENABLE_PROFILER: "true"
    SANDBOX_PROFILE: "ran-optimizer"
  
  resources:
    gpu: true
    memory: 32Gi
    cpu: 8
```

#### 7.2 RAN Data Pipeline

```bash
# Mount ENM PM export data into the sandbox
# This runs on the host, feeding data to the sandboxed agent

#!/bin/bash
# sync-enm-data.sh — Sync Ericsson ENM exports to sandbox data volume

ENM_HOST="${ENM_HOST:-enm-prod.ericsson.local}"
SANDBOX_NAME="${1:-ericsson-ran-optimizer}"
DATA_DIR="/data/enm-exports"

# PM counters (15-min granularity)
rsync -avz "${ENM_HOST}:/var/log/enmcollector/pm/" "${DATA_DIR}/pm/"

# FM alarms
rsync -avz "${ENM_HOST}:/var/log/enmcollector/fm/" "${DATA_DIR}/fm/"

# CM baseline
rsync -avz "${ENM_HOST}:/var/log/enmcollector/cm/" "${DATA_DIR}/cm/"

# Copy to sandbox data volume
openshell sandbox exec "$SANDBOX_NAME" -- \
    cp -r /host-data/pm/ /data/pm/
```

**Phase 7 Exit Criteria**:
- [ ] RAN optimizer sandbox starts with Ericsson-specific env vars and data mounts
- [ ] Agent can query ruvector-postgres for historical KPI data
- [ ] GNN-based cell topology model loads via ruvector-nervous-system
- [ ] 15-minute PM counter data flows into the sandbox from ENM exports
- [ ] KPI evaluation harness runs against target thresholds

---

## 6. Risk Register

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| OpenShell stays alpha for months | High | Medium | Build abstraction layer; easy to swap runtime later |
| macOS Docker has no GPU passthrough | Certain | High | MLX inference on host, route via OpenShell inference endpoint; RVF kernel boot only on Linux cloud nodes |
| NemoClaw API changes break integration | Medium | Low | We target OpenShell directly; NemoClaw is optional |
| crates.io rate limits block discovery | Low | Low | Cache results, poll weekly, use HF scheduled jobs |
| SkyPilot spot preemption during fleet spawn | Medium | Medium | Use SkyPilot managed jobs with auto-retry; checkpoint sandbox state |
| RVF self-boot needs Linux kernel | Certain | Medium | WASM/userspace tier on Mac; kernel boot reserved for Linux cloud nodes |
| Sandbox image size > 10GB | Medium | Medium | Multi-stage Docker builds; profile-specific images (not full-stack) |

---

## 7. Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Time to spawn single sandbox | < 30 seconds | `time openshell sandbox create --from rvf-sandbox:ran-optimizer` |
| Time to spawn 10-sandbox fleet | < 5 minutes | `time sandbox-factory spawn fleet-manifest.yaml` |
| Sandbox image build time (per profile) | < 15 minutes | CI pipeline timing |
| Crate discovery → image rebuild cycle | < 1 hour | HF Jobs execution time |
| Inference latency (local MLX) | < 500ms first token | OpenShell inference metrics |
| Inference latency (NVIDIA cloud) | < 2s first token | OpenShell inference metrics |
| Fleet uptime (local always-on agents) | > 99% | Health monitor reporting |
| Cloud burst cost per GPU-hour | < $2/hr (spot) | SkyPilot cost optimizer output |

---

## 8. Phase Summary + Dependencies

```
Phase 1 (Week 1)     Phase 2 (Week 2)     Phase 3 (Week 3-4)
Base Image +          Fleet Spawner +       SkyPilot + Multi-Cloud +
OpenShell Validate    Manifest Format       On-Prem Hybrid
    │                     │                     │
    └──────┬──────────────┘                     │
           │                                    │
    Phase 4 (Week 5)     Phase 5 (Week 6)      │
    Inference Routing    Crate Auto-Discovery   │
           │                     │              │
           └─────────┬───────────┘              │
                     │                          │
              Phase 6 (Week 7)                  │
              Observability                     │
                     │                          │
                     └──────────┬───────────────┘
                                │
                         Phase 7 (Week 8-9)
                         RAN-Specific Integration
```

**Total estimated duration**: 9 weeks to full production capability.  
**MVP (Phases 1-2)**: 2 weeks — local sandbox fleet with declarative manifests.  
**Cloud-ready (Phases 1-4)**: 5 weeks — hybrid local + cloud with inference routing.

---

## 9. Repository Layout

```
agent-sandbox-factory/
├── README.md
├── sandbox-factory                    # CLI entrypoint (bash)
├── spawn-fleet.py                     # Fleet spawner
├── discover-crates.py                 # Crate discovery pipeline
├── fleet-health.py                    # Health monitoring
│
├── sandboxes/
│   └── rvf-base/
│       ├── Dockerfile                 # Multi-stage, profile-parameterized
│       ├── boot-rvf.sh                # Sandbox entrypoint
│       ├── install-profile.sh         # Crate installer
│       ├── policy.yaml                # Default sandbox policy
│       ├── skills/
│       │   └── rvf-tools/
│       │       └── SKILL.md
│       └── profiles/
│           ├── base.toml
│           ├── audited-reasoner.toml
│           ├── ran-optimizer.toml
│           ├── swarm-mesh.toml
│           ├── content-publisher.toml
│           ├── persistent-memory.toml
│           ├── benchmark-eval.toml
│           ├── microvm-sealed.toml
│           ├── edge-wasm-agent.toml
│           ├── wifi-sensing.toml
│           ├── neural-inference.toml
│           └── full-stack.toml
│
├── manifests/
│   ├── fleet-manifest-local.yaml      # Local Mac Studio fleet
│   ├── fleet-manifest-cloud.yaml      # Cloud-only fleet
│   └── fleet-manifest-hybrid.yaml     # Hybrid topology
│
├── agents/
│   ├── ran-optimizer.yaml             # Per-agent detailed spec
│   ├── content-publisher.yaml
│   └── audited-reasoner.yaml
│
├── policies/
│   ├── baseline.yaml                  # Default policy
│   ├── strict.yaml                    # Locked-down policy
│   └── inference-hybrid.yaml          # Inference routing config
│
├── skypilot/
│   ├── single-node-fleet.yaml         # Single GPU node
│   ├── multi-node-fleet.yaml          # Distributed fleet
│   └── crate-discovery.yaml           # Scheduled discovery job
│
├── rvf-images/                        # Pre-built .rvf cognitive containers
│   ├── ran-optimizer.rvf
│   ├── content-agent.rvf
│   └── mesh-agent.rvf
│
└── scripts/
    ├── sync-enm-data.sh               # Ericsson data sync
    ├── build-all-profiles.sh           # Build all Docker images
    └── test-fleet.sh                   # Integration test suite
```