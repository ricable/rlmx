# ADR-004: Tiered Model Inference with Automatic Escalation

## Status
Proposed

## Date
2026-03-18

## Context

The current `rlmx-ruvllm` crate (`crates/rlmx-ruvllm/src/engine.rs`) wraps a single `CandleBackend` for local GGUF model inference. The `LocalEngine` struct provides `load_model()` and `generate()` methods, and the `ModelManager` discovers GGUF files in `~/.rlmx/models/`. This design has limitations:

1. **Single model tier**: Only small quantized models (0.5B-1.1B Q4) are supported locally. There is no path to medium-sized models (3B-8B) that benefit from Metal/MLX acceleration.
2. **No escalation**: If the edge model produces low-confidence output, the system cannot automatically escalate to a more capable model or remote backend.
3. **No MLX support**: Apple Silicon nodes (Mac Studio M2 Ultra in Zone A) have 192 GB unified memory, capable of running 70B models via MLX. The Candle backend does not leverage MLX's Metal kernel optimizations.
4. **Missing Claude Code integration**: The CLAUDE.md 3-tier model routing (ADR-026) defines Agent Booster (<1ms), Haiku (~500ms), and Sonnet/Opus (2-5s) tiers. The inference engine has no concept of these tiers.

The three inference paths described in the project overview (Cloud/Dev via `VllmClient`, Edge via `LocalEngine`, Browser via `@ruvector/ruvllm-wasm`) operate independently with no unified tier abstraction.

## Decision

Implement **tiered model inference** in `rlmx-ruvllm` with four model tiers, three backends, and confidence-based automatic escalation.

### Model Tiers

```rust
// crates/rlmx-ruvllm/src/tier.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelTier {
    /// 0.5B-1.1B Q4 GGUF. RPi5/RPi4/browser. <500ms latency.
    Small,
    /// Routed to Claude Code Agent Booster (WASM). <1ms. Simple transforms only.
    ClaudeCode,
    /// 3B-8B via MLX on Apple Silicon. Mac Studio only. 1-3s latency.
    Medium,
    /// Custom model with explicit backend and parameters.
    Custom {
        model_name: String,
        backend: InferenceBackend,
        max_tokens: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InferenceBackend {
    /// Candle GGUF (CPU/Metal/CUDA). Existing LocalEngine path.
    Candle,
    /// MLX via Python subprocess. Apple Silicon only.
    Mlx,
    /// Remote HTTP endpoint (vLLM, OpenAI-compatible).
    Remote { endpoint: String },
}
```

### TieredEngine

```rust
// crates/rlmx-ruvllm/src/tiered.rs
pub struct TieredEngine {
    pub candle: Option<LocalEngine>,           // existing, feature-gated
    pub mlx: Option<MlxSubprocess>,            // new, Apple Silicon only
    pub remote: Option<VllmClient>,            // from rlmx-rlm crate
    pub escalation_config: EscalationConfig,
}

pub struct EscalationConfig {
    /// Confidence threshold below which escalation triggers.
    pub confidence_threshold: f32,             // default: 0.4
    /// Maximum escalation chain length.
    pub max_escalations: usize,                // default: 2
    /// Tier escalation order.
    pub chain: Vec<ModelTier>,                 // default: [Small, Medium, Remote]
    /// Timeout per tier before escalating.
    pub tier_timeout: Duration,                // default: 5s
}
```

When a `generate()` call completes, the `TieredEngine` evaluates the output confidence (perplexity-based for local models, logprob-based for remote). If confidence falls below `confidence_threshold`, it escalates to the next tier in the chain.

```rust
impl TieredEngine {
    pub async fn generate_tiered(
        &self,
        prompt: &str,
        tier: ModelTier,
    ) -> Result<TieredOutput, RuvllmError> {
        let mut current_tier = tier;
        let mut escalation_count = 0;

        loop {
            let result = self.generate_at_tier(prompt, &current_tier).await?;

            if result.confidence >= self.escalation_config.confidence_threshold
                || escalation_count >= self.escalation_config.max_escalations
            {
                return Ok(result);
            }

            match self.next_tier(&current_tier) {
                Some(next) => {
                    tracing::info!(
                        from = ?current_tier, to = ?next,
                        confidence = result.confidence,
                        "Escalating to higher tier"
                    );
                    current_tier = next;
                    escalation_count += 1;
                }
                None => return Ok(result), // no higher tier available
            }
        }
    }
}
```

### MLX Subprocess Bridge

For Medium tier on Apple Silicon, `MlxSubprocess` bridges to Python MLX:

```rust
// crates/rlmx-ruvllm/src/mlx_bridge.rs
pub struct MlxSubprocess {
    child: Option<tokio::process::Child>,
    stdin: tokio::process::ChildStdin,
    stdout: BufReader<tokio::process::ChildStdout>,
    model_name: String,
}

impl MlxSubprocess {
    pub async fn spawn(model_name: &str) -> Result<Self, RuvllmError> {
        let child = tokio::process::Command::new("python3")
            .arg("deploy/mlx_bridge.py")
            .arg("--model").arg(model_name)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        // ... setup stdin/stdout handles
    }

    pub async fn generate(&mut self, request: MlxRequest) -> Result<MlxResponse, RuvllmError> {
        let json = serde_json::to_string(&request)?;
        self.stdin.write_all(json.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        let mut line = String::new();
        self.stdout.read_line(&mut line).await?;
        Ok(serde_json::from_str(&line)?)
    }
}
```

`deploy/mlx_bridge.py` wraps `autoresearch-mlx` for training and `mlx-lm` for inference via JSON-line protocol (stdin/stdout). It runs as a long-lived subprocess; model loading happens once at startup.

### MCP Edge Tool Extensions

- `rlmx_edge_generate`: gains optional `tier` parameter (default: `Small`). Operator+ role.
- `rlmx_edge_status`: reports all available tiers and their health. Viewer+ role.
- `rlmx_edge_load`: gains `backend` parameter to specify `Candle`, `Mlx`, or `Remote`. Engineer+ role.

### Escalation Flow

`TinyDancerRouter` selects `Strategy::Edge`, which enters `TieredEngine`. The engine tries Small (Candle GGUF, RPi5), and if confidence < 0.4, escalates to Medium (MLX, Mac Studio), then to Remote (vLLM cloud). Each tier has a 5s timeout before escalation.

## Consequences

### Positive
- Automatic quality improvement: low-confidence outputs escalate without user intervention
- MLX support unlocks 70B models on Mac Studio hardware already in the cluster
- Unified interface: `TieredEngine::generate_tiered()` abstracts all three backends
- Backward compatible: existing `LocalEngine` continues to work as the `Candle` backend

### Negative
- Python subprocess for MLX adds a Python runtime dependency on Zone A Mac nodes
- Escalation adds latency: worst case Small (500ms) + Medium (3s) + Remote (5s) = 8.5s
- Confidence estimation varies by backend (perplexity vs logprob), complicating threshold tuning

### Risks
- MLX subprocess crash requires restart; no shared state with Rust process
- Remote backend dependency means escalation fails if cloud is unreachable
- Perplexity-based confidence for small GGUF models is unreliable for out-of-distribution inputs

## Alternatives Considered

1. **Single model per node**: Each node runs exactly one model with no escalation. Rejected because it wastes Zone A hardware (Mac Studio running only 1.1B models) and provides no quality improvement path for hard queries.

2. **External routing service**: A separate microservice decides which model to use before inference. Rejected because it adds network round-trip latency and a single point of failure. The `TinyDancerRouter` (ADR-003) already runs in-process.

3. **Always-cloud inference**: Route all queries to cloud vLLM. Rejected because it eliminates the edge deployment use case, increases cost, and adds latency for queries that small models handle well.

4. **Candle MLX backend**: Use Candle's experimental Metal backend instead of MLX subprocess. Rejected because Candle's Metal support is limited to specific operations, while MLX has full Apple Silicon optimization including fused attention kernels.

## References
- `crates/rlmx-ruvllm/src/engine.rs` -- `LocalEngine`, `CandleBackend` wrapper
- `crates/rlmx-ruvllm/src/manager.rs` -- `ModelManager` for GGUF discovery in `~/.rlmx/models/`
- `crates/rlmx-rlm/` -- `VllmClient` for remote OpenAI-compatible inference
- `crates/rlmx-mcp/src/tools.rs` -- edge tools: `rlmx_edge_generate`, `rlmx_edge_status`, `rlmx_edge_load`
- `deploy/rlmx-edge.service` -- systemd unit for RPi5, will be extended for MLX nodes
- ADR-003 -- `TinyDancerRouter` selects `Strategy::Edge` which triggers tiered inference
